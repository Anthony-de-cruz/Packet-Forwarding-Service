# Packet Forwarding Service

## Dependencies

This project is dependent on [rust-pcap](https://github.com/rust-pcap/pcap/tree/main) and in turn, [libpcap](https://github.com/the-tcpdump-group/libpcap).

### Windows

1. Install [Npcap](https://npcap.com/#download).
2. Download the [Npcap SDK](https://npcap.com/#download).
3. Add the SDK's `/Lib` or `/Lib/x64` folder to your `LIB` environment variable.

### Linux

Install the libraries and header files for the libpcap library. For example:

- On Debian based Linux: install `libpcap-dev`.
- On Fedora Linux: install `libpcap-devel`.
- On NixOS: use the provided flake via `nix flake update && nix develop`.

Build/run with `sudo -E cargo run` to maintain dynamic library link.

### Mac OS X

`libpcap` should be installed on Mac OS X by default.
