# Packet Forwarding Service

## Dependencies

The router is dependent on:

- Root level accces.
- Kernel modules: `br_netfilter`, `nfnetlink_queue`, `nf_conntrack`
- [Packet-Classifier](https://github.com/Anthony-de-cruz/Packet-Classifier) to provide a trained CNN for traffic classification.
- [ort](https://github.com/pykeio/ort) to provide a Rust runtime for the trained model.
- [libnetfilter_queue](https://netfilter.org/projects/libnetfilter_queue/index.html) library and the corresponding Rust binding [nfq crate](https://github.com/nbdd0121/nfq-rs).
- [crossbeam-channel crate](https://github.com/crossbeam-rs/crossbeam/tree/master) Thread-safe channel implementation.

To install router dependencies:

Run `git submodule init && git submodule update` to fetch Packet-Classifier. Perform any required instructions within that repo.

- On Debian based Linux: install `libnetfilter-queue-dev`.
- On Fedora Linux: install `libnetfilter_queue`.
- Via Nix: use the provided flake via `nix develop`.

Associated forwarding nodes are dependent on:

- Root level access.

Associated traffic generation is dependent on either:

- [quickemu]() + `Ubuntu 22.04` (or alternative VM) to provide an environment for [Open5GS]() and [UERANSIM]().
- Kernel module `tun` for bridging.

Or:

- The provided `python3` and `iperf3` scripts.

The intended setup includes 2 Ubuntu 22.04 VMs to simulate a 5G cellular network.
However, 2 Python scripts are provided to simulate network traffic by
transmitting packet samples via UDP. This can act as a significantly simpler interim demonstrator.
The next level would be to set up an `iperf3` server/client to simulate a heavy traffic load.

## Setup

> [!IMPORTANT]
> Read the scripts found in `scripts/` understand what is going on before you run anything.

To setup VMs:

- On Debian or Fedora based Linux: follow official instructions [here](https://github.com/quickemu-project/quickemu/wiki/01-Installation) to install `quickemu`.
- Via Nix: use the provided flake via `nix develop`.

Then run:

```sh
./scripts/vm-init-env.sh
```

Once completed, go into the VMs and setup Open5GS and UERANSIM as normal.

In the Open5GS VM, follow the [quickstart guide](https://open5gs.org/open5gs/docs/guide/01-quickstart/). Then run:

```sh
./scripts/vm-open5gs-init-env.sh 
./scripts/vm-open5gs-restart.sh 
```

In the UERANSIM VM, follow the [installation guide](https://github.com/aligungr/UERANSIM/wiki/Installation). Then run:

```sh
./scripts/vm-ueransim-init-env.sh
```

To setup server host:

```sh
./scripts/router-init-env.sh
```

On the forwarding nodes:

```sh
./scripts/node-init-env.sh
```

To build/run the router with `sudo -E cargo run` to maintain dynamic library link.
