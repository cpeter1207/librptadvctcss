/** @file
 * @brief Conditioned 8 kHz CTCSS receive and 48 kHz transmit descriptor ABI.
 */
#ifndef RPTADV_CTCSS_H
#define RPTADV_CTCSS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define RPTADV_CTCSS_ABI_VERSION 2U /**< Status/out-handle and u32-boolean ABI. */
#define RPTADV_CTCSS_OK 0 /**< Operation succeeded. */
#define RPTADV_CTCSS_INVALID_ARGUMENT -1 /**< Required out pointer is null. */

/** @brief Opaque state for one conditioned 8 kHz receive stream. */
struct rptadv_ctcss_rx;
/** @brief Opaque phase state for one 48 kHz transmit stream. */
struct rptadv_ctcss_tx;

/** @brief Immutable function table for the process lifetime. */
struct rptadv_ctcss_descriptor {
  uint32_t struct_size; /**< Size of this descriptor. */
  uint32_t abi_version; /**< Must equal RPTADV_CTCSS_ABI_VERSION. */
  const char *capability_name; /**< Stable `rptadv.ctcss` identity. */

  /** @brief Allocate receive state during setup.
   * @param out Required output slot; cleared before allocation.
   * @return RPTADV_CTCSS_OK or RPTADV_CTCSS_INVALID_ARGUMENT.
   */
  int32_t (*rx_create)(struct rptadv_ctcss_rx **out);
  /** @brief Release receive state after its callback has stopped. Null is accepted. */
  void (*rx_destroy)(struct rptadv_ctcss_rx *handle);
  /** @brief Reset detector state for selected table entries and relax mode.
   * @param relax Zero is false; any nonzero value is true.
   */
  void (*rx_configure)(struct rptadv_ctcss_rx *handle, uint64_t tone_mask,
                       uint32_t relax);
  /** @brief Return 1 if any supported table entry is selected, else 0. */
  uint32_t (*rx_enabled)(const struct rptadv_ctcss_rx *handle);
  /** @brief Return the selected table index, or -1 when unqualified. */
  int16_t (*rx_decoded)(const struct rptadv_ctcss_rx *handle);
  /** @brief Return 1 if a 55 Hz release tail is active, else 0. */
  uint32_t (*rx_tail_tone_active)(const struct rptadv_ctcss_rx *handle);
  /** @brief Clear tail state only when carrier is absent (zero). */
  void (*rx_clear_tail_on_carrier_loss)(struct rptadv_ctcss_rx *handle,
                                        uint32_t carrier);
  /** @brief Consume conditioned 8 kHz mono F32 samples; return table index or -1.
   * @param samples Readable for count values. Null is accepted only when count is zero.
   * @param count Number of samples in this callback.
   * @param carrier Current CTCSS carrier indication; zero false, nonzero true.
   */
  int16_t (*rx_process)(struct rptadv_ctcss_rx *handle, const float *samples,
                        uint32_t count, uint32_t carrier);

  /** @brief Allocate transmit phase state during setup.
   * @param out Required output slot; cleared before allocation.
   * @return RPTADV_CTCSS_OK or RPTADV_CTCSS_INVALID_ARGUMENT.
   */
  int32_t (*tx_create)(struct rptadv_ctcss_tx **out);
  /** @brief Release transmit state after its callback has stopped. Null is accepted. */
  void (*tx_destroy)(struct rptadv_ctcss_tx *handle);
  /** @brief Return the legacy fixed-increment frequency for a supported tone. */
  double (*legacy_frequency)(double frequency_hz);
  /** @brief Render 48 kHz mono F32 PCM and retain phase in the handle.
   * @param output Writable for frames values; null is accepted only when frames is zero.
   * @param frames Number of output samples.
   * @param frequency_hz Calibrated transmit frequency.
   * @param peak Linear amplitude.
   * @param enabled Nonzero generates, zero fills positive-zero silence.
   * @param phase_shift_degrees One-time phase shift applied before this span.
   */
  void (*tx_generate)(struct rptadv_ctcss_tx *handle, float *output,
                      uint32_t frames, double frequency_hz, float peak,
                      uint32_t enabled, double phase_shift_degrees);
};

/** @brief Return the immutable descriptor; validate identity, size and ABI. */
const struct rptadv_ctcss_descriptor *rptadv_ctcss_descriptor(void);

#ifdef __cplusplus
}
#endif

#endif
