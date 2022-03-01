# HyperCAN

HyperCAN is a Rust-based command line utility for communicating with OBD/UDS services over CAN. It
provides a straightforward set of subcommands and configurable flags, designed for allowing users to
quickly and easily query these services and get back semantically-relevant data.

## Caveat Emptor

HyperCAN is currently under development.  A lot of testing happens against virtual CAN interfaces,
using mocked or emulated devices on the bus.  This can mean that what appears to work in testing may
or may not work for you in real-world conditions.  Filing an issue (with simple reproduction instructions)
is always welcome.

## Requirements

HyperCAN depends on [SocketCAN][socketcan], which makes this utility Linux-only.  Sorry!  Life is
too short to wrangle support for multiple platorms, arbitrary J2534 DLLs, and all of that. :)

Additionally, HyperCAN depends on ISO-TP support.  ISO-TP support was added to the Linux mainline
kernel from 5.10 onward.  If you're running an older kernel, you can compile support for it on your
own by using the following repository: [hartkopp/can-isotp][can_isotp].  I don't have anything to do
with that kernel module, so please don't ask for support compiling it.

Other than that, HyperCAN is built against stable Rust: 1.59.0 at the time of writing.  If HyperCAN
does not build against stable Rust from 1.59.0 and newer: it's a bug, please let me know.

## Supported Features

- [x] Validate a SocketCAN interface exists and can be opened. (`validate-socket` subcommand)
- [x] Read all available OBD-II current data PIDs. (`query-available-pids` subcommand)
- [ ] Read the current data of an OBD-II PID(s). (OBD-II, Service 01)
- [ ] Read/clear stored diagnostic trouble codes. (OBD-II, Services 03 and 04)
- [ ] Read vehicle information. (OBD-II, Service 09)
- [ ] Any UDS service.


## License

HyperCAN is licensed under the MIT license.

[socketcan]: https://en.wikipedia.org/wiki/SocketCAN
[can_isotp]: https://github.com/hartkopp/can-isotp
