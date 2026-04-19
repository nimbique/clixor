# Clixor

High-performance autoclicker for Windows written in Rust.

Uses a low-level `WH_MOUSE_LL` hook to detect physical mouse input and inject
clicks at a configurable rate. Injected events are ignored to prevent feedback loops.

## Features

- Independent LMB / RMB control — separate CPS and toggle keys
- Smooth CPS ramp-up — configurable start ratio and duration
- High-resolution timer via `WaitableTimer` with fallback to `thread::sleep`
- Hybrid wait strategy — sleep + spin-loop to minimize click jitter
- JSON config — auto-generated on first run
- Clean shutdown — releases all buttons on Ctrl+C and panic

## Requirements

- Windows 10 / 11
- Rust toolchain (`rustup.rs`)
- Administrator privileges at runtime

## Build

```bash
git clone https://github.com/nimbique/clixor
cd clixor
cargo build --release
```

Binary: `target/release/Clixor.exe`

## Usage

1. Run `Clixor.exe` as Administrator
2. Hold a mouse button — clicks fire automatically
3. Press the toggle key to enable / disable each button
4. `Ctrl+C` to exit

## Configuration

`config.json` is created automatically next to the executable on first run.

```json
{
  "lmb_cps": 10.0,
  "rmb_cps": 10.0,
  "lmb_toggle_key": "F6",
  "rmb_toggle_key": "F7",
  "ramp_duration_ms": 300,
  "ramp_start_ratio": 0.20,
  "spin_threshold_us": 1500,
  "sleep_undershoot_us": 600
}
```

| Field | Description |
|---|---|
| `lmb_cps` / `rmb_cps` | Target clicks per second (1–1000) |
| `lmb_toggle_key` / `rmb_toggle_key` | Toggle keys — F1–F12, CAPS, TAB, A–Z, etc. |
| `ramp_duration_ms` | Time to reach target CPS from ramp start (ms) |
| `ramp_start_ratio` | Initial CPS as a fraction of target (0.01–1.0) |
| `spin_threshold_us` | Switch to spin-loop below this threshold (µs) |
| `sleep_undershoot_us` | Sleep margin before next click (µs); must be less than `spin_threshold_us` |

## License

MIT