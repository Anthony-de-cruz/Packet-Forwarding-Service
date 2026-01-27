# Packet Forwarding Service

## Dependencies

This project is dependent on the [libnetfilter_queue] library and the corresponding Rust binding [nfq](https://github.com/nbdd0121/nfq-rs).

- On Debian based Linux: install `libnetfilter-queue-dev`.
- On Fedora Linux: install `libnetfilter_queue`.
- On NixOS: use the provided flake via `nix flake update && nix develop`.

Build/run with `sudo -E cargo run` to maintain dynamic library link.
