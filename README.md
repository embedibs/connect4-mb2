# connect4-mb2

Connect 4 running on a tft display connected to the BBC Micro:Bit v2 through a
Chip-8 interpreter.

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

## License

This project is licensed under the [MIT License][License].

[License]: ./LICENSE
