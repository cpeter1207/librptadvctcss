/** @file
 * @brief Conditioned 8 kHz CTCSS receive and 48 kHz transmit descriptor ABI.
 */
#ifndef RPTADV_CTCSS_H
#define RPTADV_CTCSS_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define RPTADV_CTCSS_ABI_VERSION 1U /**< Initial CTCSS descriptor ABI. */

/** @brief Opaque state for one conditioned 8 kHz receive stream. */
struct rptadv_ctcss_rx;
/** @brief Opaque phase state for one 48 kHz transmit stream. */
struct rptadv_ctcss_tx;

/** @brief Immutable function table for the process lifetime. */
struct rptadv_ctcss_descriptor {
  uint32_t struct_size; /**< Size of this descriptor. */
  uint32_t abi_version; /**< Must equal RPTADV_CTCSS_ABI_VERSION. */
  const char *capability_name; /**< Stable `rptadv.ctcss` identity. */

  /** @brief Allocate receive state during setup. */
  struct rptadv_ctcss_rx *(*rx_create)(void);
  /** @brief Release receive state after its callback has stopped. Null is accepted. */
  void (*rx_destroy)(struct rptadv_ctcss_rx *handle);
  /** @brief Reset detector state for the selected table entries and relax mode. */
  void (*rx_configure)(struct rptadv_ctcss_rx *handle, uint64_t tone_mask,
                       bool relax);
  /** @brief Return whether any supported table entry is selected. */
  bool (*rx_enabled)(const struct rptadv_ctcss_rx *handle);
  /** @brief Return the selected table index, or -1 when unqualified. */
  int16_t (*rx_decoded)(const struct rptadv_ctcss_rx *handle);
  /** @brief Return whether a 55 Hz release tail is active. */
  bool (*rx_tail_tone_active)(const struct rptadv_ctcss_rx *handle);
  /** @brief Clear tail state only when carrier is absent. */
  void (*rx_clear_tail_on_carrier_loss)(struct rptadv_ctcss_rx *handle,
                                        bool carrier);
  /** @brief Consume conditioned 8 kHz mono F32 samples; return table index or -1.
   * @param samples Readable for count values. A null pointer is rejected.
   * @param count Number of samples in this callback.
   * @param carrier Current CTCSS carrier indication.
   */
  int16_t (*rx_process)(struct rptadv_ctcss_rx *handle, const float *samples,
                        uint32_t count, bool carrier);

  /** @brief Allocate transmit phase state during setup. */
  struct rptadv_ctcss_tx *(*tx_create)(void);
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
