# Packet Forwarding Service

## Dependencies

This project is dependent on:
- [Packet-Classifier](https://github.com/Anthony-de-cruz/Packet-Classifier) to provide a trained CNN for traffic classification.
- [ort](https://github.com/pykeio/ort) to run.
- [libnetfilter_queue](https://netfilter.org/projects/libnetfilter_queue/index.html) library and the corresponding Rust binding [nfq](https://github.com/nbdd0121/nfq-rs).

To install dependencies:

Run `git submodule init && git submodule update` to fetch Packet-Classifier. Perform any required instructions within that repo.

- On Debian based Linux: install `libnetfilter-queue-dev`.
- On Fedora Linux: install `libnetfilter_queue`.
- On NixOS: use the provided flake via `nix flake update && nix develop`.

Build/run with `sudo -E cargo run` to maintain dynamic library link.
