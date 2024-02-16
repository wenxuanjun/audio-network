# Audio Network

Create a virtual network interface transferring data based on audio signals, wired or wireless.

Custom MAC and CSMA (Multiple Access) is finished but test cases are removed due to messy performance.

## Build

```bash
cargo build --release
```

Remember to give the binary the necessary capability in order to create TAP interface without root permission.

```bash
sudo setcap cap_net_admin=+pe target/release/audio_network
```

Append `--help` to see the usage.

```bash
target/release/audio_network --help
```

## Usage

You should tune the `PacketDetector` before using it.

First, make sure your speakers and microphone are working well (you can connect the inputs and outputs directly with audio cables if you're testing wired mode).

Then execute the following command to run the `tune_detector` test. It will use `numpy` and `matplotlib` to plot the correlation between the input and output signals.

```bash
cargo test --test tune_detector -- --nocapture --ignored
```

Fill in the peak correlation from each preamble into `DETECT_THRETSHOLD_MIN` in `src/packet/detector.rs`.

There are some scripts in `scripts` directory to help you test the virtual interface.

## Compatibility

Linux only. Because it uses TAP interface.
