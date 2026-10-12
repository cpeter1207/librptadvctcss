#include "rptadvctcss/rptadvctcss.h"

#include <assert.h>
#include <stddef.h>
#include <string.h>

#if RPTADV_CTCSS_ABI_VERSION != 2U
#error CTCSS ABI 2 is required
#endif

int main(void) {
  const struct rptadv_ctcss_descriptor *api = rptadv_ctcss_descriptor();
  assert(api == rptadv_ctcss_descriptor());
  assert(api->struct_size == sizeof(*api));
  assert(api->abi_version == RPTADV_CTCSS_ABI_VERSION);
  assert(strcmp(api->capability_name, "rptadv.ctcss") == 0);
  assert(api->rx_create && api->rx_destroy && api->rx_configure &&
         api->rx_enabled && api->rx_decoded && api->rx_tail_tone_active &&
         api->rx_clear_tail_on_carrier_loss && api->rx_process &&
         api->tx_create && api->tx_destroy && api->legacy_frequency &&
         api->tx_generate);

  struct rptadv_ctcss_rx *rx = NULL;
  struct rptadv_ctcss_tx *tx = NULL;
  assert(api->rx_create(NULL) == RPTADV_CTCSS_INVALID_ARGUMENT);
  assert(api->tx_create(NULL) == RPTADV_CTCSS_INVALID_ARGUMENT);
  assert(api->rx_create(&rx) == RPTADV_CTCSS_OK);
  assert(api->tx_create(&tx) == RPTADV_CTCSS_OK);
  assert(rx && tx);
  api->rx_configure(rx, UINT64_C(1) << 11, 7);
  assert(api->rx_enabled(rx) == 1U);
  assert(api->rx_decoded(rx) == -1);
  assert(api->rx_tail_tone_active(rx) == 0U);
  api->rx_clear_tail_on_carrier_loss(rx, 7);
  assert(api->rx_process(rx, NULL, 0, 7) == -1);
  api->rx_destroy(rx);

  const double frequency = api->legacy_frequency(114.8);
  assert(frequency == 114.74609375);
  float whole[960] = {0};
  float split[960] = {0};
  struct rptadv_ctcss_tx *second_tx = NULL;
  assert(api->tx_create(&second_tx) == RPTADV_CTCSS_OK);
  assert(second_tx);
  api->tx_generate(tx, whole, 960, frequency, 0.25f, 1, 120.0);
  for (size_t offset = 0; offset < 960;) {
    const size_t count = 960 - offset < 137 ? 960 - offset : 137;
    api->tx_generate(second_tx, split + offset, (uint32_t)count, frequency,
                     0.25f, 1, offset == 0 ? 120.0 : 0.0);
    offset += count;
  }
  for (size_t i = 0; i < 960; ++i)
    assert(whole[i] == split[i] ||
           (whole[i] - split[i] < 0.0000001f &&
            split[i] - whole[i] < 0.0000001f));
  api->tx_destroy(second_tx);
  api->tx_destroy(tx);
  rx = NULL;
  assert(api->rx_create(&rx) == RPTADV_CTCSS_OK);
  assert(rx);
  api->rx_configure(rx, UINT64_C(1) << 11, 0);
  assert(api->rx_enabled(rx) == 1U);
  api->rx_destroy(rx);
  api->rx_destroy(NULL);
  api->tx_destroy(NULL);
  return 0;
}
