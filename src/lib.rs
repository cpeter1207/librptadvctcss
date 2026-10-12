//! Portable CTCSS receive and transmit state behind one versioned descriptor.
//!
//! The detector consumes conditioned 8 kHz mono PCM. The generator produces
//! 48 kHz mono PCM. Each handle owns its state and is created outside callbacks.

#![deny(warnings)]
#![cfg_attr(coverage, feature(coverage_attribute))]

use std::{ffi::c_char, mem::size_of, ptr};

mod ctcss_receive;
mod ctcss_transmit;

const ABI_VERSION: u32 = 2;
const OK: i32 = 0;
const INVALID_ARGUMENT: i32 = -1;
const CAPABILITY_NAME: &[u8] = b"rptadv.ctcss\0";

/// Opaque state for one conditioned CTCSS receive stream.
#[derive(Default)]
pub struct CtcssRx(ctcss_receive::ReceiveState);

/// Opaque phase state for one CTCSS transmit stream.
#[derive(Default)]
pub struct CtcssTx {
    phase: f64,
}

/// ABI 2 CTCSS function table.
#[repr(C)]
pub struct CtcssDescriptor {
    struct_size: u32,
    abi_version: u32,
    capability_name: *const c_char,
    rx_create: unsafe extern "C" fn(*mut *mut CtcssRx) -> i32,
    rx_destroy: unsafe extern "C" fn(*mut CtcssRx),
    rx_configure: unsafe extern "C" fn(*mut CtcssRx, u64, u32),
    rx_enabled: unsafe extern "C" fn(*const CtcssRx) -> u32,
    rx_decoded: unsafe extern "C" fn(*const CtcssRx) -> i16,
    rx_tail_tone_active: unsafe extern "C" fn(*const CtcssRx) -> u32,
    rx_clear_tail_on_carrier_loss: unsafe extern "C" fn(*mut CtcssRx, u32),
    rx_process: unsafe extern "C" fn(*mut CtcssRx, *const f32, u32, u32) -> i16,
    tx_create: unsafe extern "C" fn(*mut *mut CtcssTx) -> i32,
    tx_destroy: unsafe extern "C" fn(*mut CtcssTx),
    legacy_frequency: extern "C" fn(f64) -> f64,
    tx_generate: unsafe extern "C" fn(*mut CtcssTx, *mut f32, u32, f64, f32, u32, f64),
}

// SAFETY: this descriptor and all addresses it references are immutable.
unsafe impl Sync for CtcssDescriptor {}

static DESCRIPTOR: CtcssDescriptor = CtcssDescriptor {
    struct_size: size_of::<CtcssDescriptor>() as u32,
    abi_version: ABI_VERSION,
    capability_name: CAPABILITY_NAME.as_ptr().cast::<c_char>(),
    rx_create,
    rx_destroy,
    rx_configure,
    rx_enabled,
    rx_decoded,
    rx_tail_tone_active,
    rx_clear_tail_on_carrier_loss,
    rx_process,
    tx_create,
    tx_destroy,
    legacy_frequency,
    tx_generate,
};

/// Return the immutable ABI 2 function table.
#[unsafe(no_mangle)]
pub extern "C" fn rptadv_ctcss_descriptor() -> *const CtcssDescriptor {
    &DESCRIPTOR
}

unsafe extern "C" fn rx_create(out: *mut *mut CtcssRx) -> i32 {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return INVALID_ARGUMENT;
    };
    *out = ptr::null_mut();
    *out = Box::into_raw(Box::new(CtcssRx::default()));
    OK
}

unsafe extern "C" fn rx_destroy(handle: *mut CtcssRx) {
    if !handle.is_null() {
        // SAFETY: ownership returns once after receive callbacks have stopped.
        drop(unsafe { Box::from_raw(handle) });
    }
}

unsafe extern "C" fn rx_configure(handle: *mut CtcssRx, tone_mask: u64, relax: u32) {
    // SAFETY: the caller holds this receive handle exclusively.
    if let Some(state) = unsafe { handle.as_mut() } {
        state.0.configure(tone_mask, relax != 0);
    }
}

unsafe extern "C" fn rx_enabled(handle: *const CtcssRx) -> u32 {
    // SAFETY: the caller owns the handle for the duration of this call.
    u32::from(unsafe { handle.as_ref() }.is_some_and(|state| state.0.enabled()))
}

unsafe extern "C" fn rx_decoded(handle: *const CtcssRx) -> i16 {
    // SAFETY: the caller owns the handle for the duration of this call.
    unsafe { handle.as_ref() }.map_or(-1, |state| state.0.decoded())
}

unsafe extern "C" fn rx_tail_tone_active(handle: *const CtcssRx) -> u32 {
    // SAFETY: the caller owns the handle for the duration of this call.
    u32::from(unsafe { handle.as_ref() }.is_some_and(|state| state.0.tail_tone_active()))
}

unsafe extern "C" fn rx_clear_tail_on_carrier_loss(handle: *mut CtcssRx, carrier: u32) {
    // SAFETY: the caller holds this receive handle exclusively.
    if let Some(state) = unsafe { handle.as_mut() } {
        state.0.clear_tail_on_carrier_loss(carrier != 0);
    }
}

unsafe extern "C" fn rx_process(
    handle: *mut CtcssRx,
    samples: *const f32,
    count: u32,
    carrier: u32,
) -> i16 {
    // SAFETY: the caller supplies a readable count-sized buffer and holds the
    // receive handle exclusively. A zero-length callback may have no buffer;
    // the detector still observes carrier loss on that callback.
    let Some(state) = (unsafe { handle.as_mut() }) else {
        return -1;
    };
    if count != 0 && samples.is_null() {
        return -1;
    }
    unsafe { state.0.process(samples, count, carrier != 0) }
}

unsafe extern "C" fn tx_create(out: *mut *mut CtcssTx) -> i32 {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return INVALID_ARGUMENT;
    };
    *out = ptr::null_mut();
    *out = Box::into_raw(Box::new(CtcssTx::default()));
    OK
}

extern "C" fn legacy_frequency(frequency_hz: f64) -> f64 {
    ctcss_transmit::legacy_frequency(frequency_hz)
}

unsafe extern "C" fn tx_destroy(handle: *mut CtcssTx) {
    if !handle.is_null() {
        // SAFETY: ownership returns once after transmit callbacks have stopped.
        drop(unsafe { Box::from_raw(handle) });
    }
}

unsafe extern "C" fn tx_generate(
    handle: *mut CtcssTx,
    output: *mut f32,
    frames: u32,
    frequency_hz: f64,
    peak: f32,
    enabled: u32,
    phase_shift_degrees: f64,
) {
    // SAFETY: the caller supplies a writable frame-sized buffer and holds the
    // transmit handle exclusively. Zero frames may use a null output pointer.
    let Some(state) = (unsafe { handle.as_mut() }) else {
        return;
    };
    if frames != 0 && output.is_null() {
        return;
    }
    unsafe {
        ctcss_transmit::generate(
            &mut state.phase,
            output,
            frames,
            frequency_hz,
            peak,
            enabled,
            phase_shift_degrees,
        )
    };
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn descriptor_identifies_abi_and_rejects_invalid_buffers() {
        let api = unsafe { &*rptadv_ctcss_descriptor() };
        assert_eq!(
            api.struct_size as usize,
            std::mem::size_of::<CtcssDescriptor>()
        );
        assert_eq!(api.abi_version, 2);
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(api.capability_name) }.to_bytes(),
            b"rptadv.ctcss"
        );
        let mut rx = ptr::null_mut();
        let mut tx = ptr::null_mut();
        assert_eq!(
            unsafe { (api.rx_create)(ptr::null_mut()) },
            INVALID_ARGUMENT
        );
        assert_eq!(
            unsafe { (api.tx_create)(ptr::null_mut()) },
            INVALID_ARGUMENT
        );
        assert_eq!(unsafe { (api.rx_create)(&mut rx) }, OK);
        assert_eq!(unsafe { (api.tx_create)(&mut tx) }, OK);
        assert!(!rx.is_null() && !tx.is_null());
        unsafe {
            (api.rx_configure)(rx, 1 << 11, 7);
            assert_eq!((api.rx_enabled)(rx), 1);
            assert_eq!((api.rx_tail_tone_active)(rx), 0);
            (api.rx_clear_tail_on_carrier_loss)(rx, 7);
            assert_eq!((*rx).0.test_detector_decode(11), 0);
            assert_eq!((api.rx_process)(rx, std::ptr::null(), 1, 7), -1);
            let sample = [0.0_f32];
            assert_eq!((api.rx_process)(rx, sample.as_ptr(), 1, 7), -1);
            assert_eq!((api.rx_decoded)(rx), -1);
            (api.tx_generate)(tx, std::ptr::null_mut(), 1, 100.0, 0.25, 1, 0.0);
            (api.tx_generate)(tx, std::ptr::null_mut(), 0, 100.0, 0.25, 1, 0.0);
            (api.rx_configure)(std::ptr::null_mut(), 1, 0);
            assert_eq!((api.rx_enabled)(std::ptr::null()), 0);
            assert_eq!((api.rx_decoded)(std::ptr::null()), -1);
            assert_eq!((api.rx_tail_tone_active)(std::ptr::null()), 0);
            (api.rx_clear_tail_on_carrier_loss)(std::ptr::null_mut(), 0);
            assert_eq!(
                (api.rx_process)(std::ptr::null_mut(), std::ptr::null(), 1, 7),
                -1
            );
            (api.tx_generate)(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                1,
                100.0,
                0.25,
                1,
                0.0,
            );
            (api.rx_destroy)(rx);
            (api.tx_destroy)(tx);
            (api.rx_destroy)(std::ptr::null_mut());
            (api.tx_destroy)(std::ptr::null_mut());
        }
    }

    #[test]
    fn independent_transmit_handles_match_chunked_output() {
        let api = unsafe { &*rptadv_ctcss_descriptor() };
        let mut whole = ptr::null_mut();
        let mut split = ptr::null_mut();
        assert_eq!(unsafe { (api.tx_create)(&mut whole) }, OK);
        assert_eq!(unsafe { (api.tx_create)(&mut split) }, OK);
        let frequency = (api.legacy_frequency)(114.8);
        assert_eq!(frequency, 114.74609375);
        let mut whole_pcm = [0.0_f32; 960];
        let mut split_pcm = [0.0_f32; 960];
        unsafe {
            (api.tx_generate)(
                whole,
                whole_pcm.as_mut_ptr(),
                960,
                frequency,
                0.25,
                1,
                120.0,
            );
            for (index, block) in split_pcm.chunks_mut(137).enumerate() {
                (api.tx_generate)(
                    split,
                    block.as_mut_ptr(),
                    block.len() as u32,
                    frequency,
                    0.25,
                    1,
                    if index == 0 { 120.0 } else { 0.0 },
                );
            }
            (api.tx_destroy)(whole);
            (api.tx_destroy)(split);
        }
        for (whole, split) in whole_pcm.into_iter().zip(split_pcm) {
            assert!((whole - split).abs() < 1e-7);
        }
    }

    #[test]
    fn zero_length_null_receive_callback_clears_tail_on_carrier_loss() {
        let api = unsafe { &*rptadv_ctcss_descriptor() };
        let mut rx = ptr::null_mut();
        assert_eq!(unsafe { (api.rx_create)(&mut rx) }, OK);
        let mut selected = [0.0_f32; 8_000];
        let mut tail = [0.0_f32; 256];
        for (index, sample) in selected.iter_mut().enumerate() {
            let phase = 2.0 * core::f32::consts::PI * 100.0 * index as f32 / 8_000.0;
            *sample = if phase.sin() >= 0.0 { 625.0 } else { -625.0 } / 32_768.0;
        }
        for (index, sample) in tail.iter_mut().enumerate() {
            let phase = 2.0 * core::f32::consts::PI * 55.0 * index as f32 / 8_000.0;
            *sample = if phase.sin() >= 0.0 { 625.0 } else { -625.0 } / 32_768.0;
        }
        unsafe {
            (api.rx_configure)(rx, 1 << 11, 0);
            for block in selected.chunks(160) {
                (api.rx_process)(rx, block.as_ptr(), block.len() as u32, 1);
            }
            assert_eq!((api.rx_decoded)(rx), 11);
            assert_eq!(
                (api.rx_process)(rx, tail.as_ptr(), tail.len() as u32, 1),
                -1
            );
            assert_eq!((api.rx_tail_tone_active)(rx), 1);
            assert_eq!((api.rx_process)(rx, std::ptr::null(), 0, 7), -1);
            assert_eq!((api.rx_tail_tone_active)(rx), 1);
            assert_eq!((api.rx_process)(rx, std::ptr::null(), 0, 0), -1);
            assert_eq!((api.rx_tail_tone_active)(rx), 0);
            (api.rx_destroy)(rx);
        }
    }
}
