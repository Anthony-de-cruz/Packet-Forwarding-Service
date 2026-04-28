# Packet Forwarding Service

## Dependencies

This server is dependent on:

- Root level accces.
- [Packet-Classifier](https://github.com/Anthony-de-cruz/Packet-Classifier) to provide a trained CNN for traffic classification.
- [ort](https://github.com/pykeio/ort) to provide a Rust runtime for the trained model.
- [libnetfilter_queue](https://netfilter.org/projects/libnetfilter_queue/index.html) library and the corresponding Rust binding [nfq crate](https://github.com/nbdd0121/nfq-rs).
- [crossbeam-channel crate](https://github.com/crossbeam-rs/crossbeam/tree/master) Thread-safe channel implementation.

Associated forwarding nodes will require/benefit from:

- Root level access.
- `tcpdump` to show traffic flows.

Associated traffic generation is dependent on either:

- `quickemu` + `Ubuntu 22.04` to provide an environment for:
- `Open5gs` + `UERANSIM`.

Or:

- The provided `Python3` script.
- `iperf3` with the provided script.

The intended setup includes 2 Ubuntu 22.04 VMs to simulate a 5G cellular network.
However, 2 Python scripts are provided to simulate network traffic by
transmitting packet samples via UDP. This can act as a significantly simpler interim demonstrator.
The next level would be to set up an `iperf3` server/client to simulate a heavy traffic load.

To install server dependencies:

Run `git submodule init && git submodule update` to fetch Packet-Classifier. Perform any required instructions within that repo.

- On Debian based Linux: install `libnetfilter-queue-dev`.
- On Fedora Linux: install `libnetfilter_queue`.
- Via Nix: use the provided flake via `nix develop`.

## Setup

> [!IMPORTANT]
> Read the scripts found in `scripts/` understand what is going on before you run anything.

To setup VMs:

- On Debian or Fedora based Linux: follow official instructions [here](https://github.com/quickemu-project/quickemu/wiki/01-Installation) to install `quickemu`.
- Via Nix: use the provided flake via `nix develop`.

Then run:

```sh
cd ./scripts
./init-vm-environ.sh
```

Once completed, go into the VMs and setup Open5GS and UERANSIM as normal.

To setup server host:

```sh
cd ./scripts
./init-router-environ.sh
```

On the forwarding nodes:

```sh
cd ./scripts
./init-node-environ.sh
```

To build/run the router with `sudo -E cargo run` to maintain dynamic library link.
