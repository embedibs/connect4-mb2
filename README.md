# connect4-mb2

Connect 4 running on a tft display connected to the BBC Micro:Bit v2 through a
Chip-8 interpreter.

## gc9a01 TFT With the BBC Micro:Bit V2

|MB2|Edge|TFT|
|-|-|-|
|p0_09|P09|RST|
|p0_10|P08|DC|
|p0_12|P01|CS|
|p0_17|P13|SCL|
|p0_13|P15|SDA|

## Dependencies

```
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools
cargo install cargo-binutils
cargo install --locked probe-rs-tools
```

## Build and Run

```
cargo embed
```

## Acknowledgements

Bart Massey [mb2-tft-display](https://github.com/pdx-cs-rust-embedded/mb2-tft-display) skeleton code

## License

This project is licensed under the [MIT License][License].

[License]: ./LICENSE
