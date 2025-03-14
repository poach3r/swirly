# Swirly
Swirly is a Rust-based desktop shell for the Sway compositor.

![demo](demo.png)

## Dependencies
**NOTE: THIS IS NOT A COMPREHENSIVE LIST, CREATE AN ISSUE IF A DEPENDENCY IS NOT LISTED HERE.**
sway

gtk4

gcc

pulseaudio

brightnessctl (optional fallback brightness control)

## Configuration
Configuration files are stored within `XDG_CONFIG_HOME/swirly/` or `HOME/.config/swirly/`.
Currently, the only configuration option is setting overrides for dock icons which is done via `overrides.toml`.

Example:
```toml
original_names = ["dev.zed.Zed", "zen"]
replacement_names = ["lite", "browser"]
```
