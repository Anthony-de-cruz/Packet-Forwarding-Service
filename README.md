# Packet Forwarding Service

## Dependencies & Setup

### Forward

This server is dependent on:

- Root level accces.
- [Packet-Classifier](https://github.com/Anthony-de-cruz/Packet-Classifier) to provide a trained CNN for traffic classification.
- [ort](https://github.com/pykeio/ort) to run.
- [libnetfilter_queue](https://netfilter.org/projects/libnetfilter_queue/index.html) library and the corresponding Rust binding [nfq](https://github.com/nbdd0121/nfq-rs).

Associated forwarding nodes will require/benefit from:

- Root level access.
- `tcpdump` to show traffic flows.

The intended setup includes 2 Ubuntu 22.04 VMs to simulate a 5G cellular network.
However, 2 Python 3 scripts are provided to simulate network traffic by
transmitting packet samples via UDP. This can act as a significantly simpler interim demonstrator.

> [!IMPORTANT]
> Read the scripts found in `scripts/` understand what is going on before you run anything.

### Setup

To setup VMs:

- On Debian or Fedora based Linux: follow official instructions [here](https://github.com/quickemu-project/quickemu/wiki/01-Installation) to install `quickemu`.
- Via Nix: use the provided flake via `nix develop`.

Then run:

```sh
cd ./scripts
sudo ./init-vm-environ.sh
```

Once completed, go into the VMs and setup Open5gs and UERANSIM as normal.

To setup server host:

```sh
sudo ./init-router-environ.sh
```

To install server dependencies:

Run `git submodule init && git submodule update` to fetch Packet-Classifier. Perform any required instructions within that repo.

- On Debian based Linux: install `libnetfilter-queue-dev`.
- On Fedora Linux: install `libnetfilter_queue`.
- Via Nix: use the provided flake via `nix develop`.

Build/run with `sudo -E cargo run` to maintain dynamic library link.
