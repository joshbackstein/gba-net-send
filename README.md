# GBA Net Send

GBA Net Send is a tool for wirelessly sending GBA homebrew to [GBA Net Boot](https://github.com/joshbackstein/gba-net-boot) on your 3DS so you can test on real hardware. Visit the GBA Net Boot repository for more information.

## Disclaimer
This software has been tested many times without any issues. However, there is always a risk something might cause damage to your system or cause corruption or loss of data. I am not responsible for any damage to or destruction of your system, your data, or anything else.

## How to use
```sh
gba-net-send <path-to-rom>
```

## How to build
Install Rust and Cargo through [rustup](https://rustup.rs/).

Then build with Cargo:
```sh
cargo build
```
