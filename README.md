# librptadvctcss

Portable CTCSS detection and generation for rpt_advanced. The runtime is
`librptadvctcss.so.2`, with ABI 2 exposed through
`rptadv_ctcss_descriptor` and `include/rptadvctcss/rptadvctcss.h`.

The algorithms, common 38-tone table, and their 27 tests are copied unchanged
from `librptadvradio/src/ctcss_receive.rs` and
`librptadvradio/src/ctcss_transmit.rs` at commit
`bfdc71738874188055169f4abb3024dafce44a67`. The descriptor, opaque
handles, and boundary tests are new. Radio-side conditioning, render state,
PTT/timer policy, and routing remain in `librptadvradio`.

Create separate receive and transmit handles during setup and destroy them
after callback use stops. Validate the descriptor identity, size, and ABI
before calling it. The receive handle consumes conditioned 8 kHz mono F32
samples and reports the selected tone's table index or `-1`. The transmit
handle renders 48 kHz mono F32 and retains oscillator phase across calls.
Processing does not allocate, lock, log, or perform I/O. Configure and handle
lifecycle belong outside audio callbacks.

ABI 2 uses `int32_t` status and an out-handle for each constructor, and
`uint32_t` booleans at the C boundary (zero is false; any nonzero input is
true; outputs are zero or one). There is no ABI 1 shim. Before upgrading,
remove the ABI 1 development package, install the ABI 2 development and
runtime packages, then rebuild and relink consumers against the ABI 2 header.
An ABI 1 descriptor must not be used as ABI 2.

Build with `make`; `make test` checks Rust behavior and the C descriptor.
`make platform-verify` adds staged install and Debian package checks. The
Debian packages are `librptadvctcss2` and `librptadvctcss-dev`, version
`0.1.0~alpha2-1` for this release. The source tag is `v0.1.0-alpha.2`.
