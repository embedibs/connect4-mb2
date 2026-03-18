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
cargo embed --release
```

## Notes

A while ago I made a [Chip-8 interpreter][OXID8] and I wanted to try making that library no-std compatible.
That's where this project came from. I also was messing around with some funny internal display stuff for the
interpreter. This project was pretty simple, mostly dropping in code from other mb2 projects and reading up
on how to use the tft-display.  

I was messing around with auto trait implementations and a trait to unpack the display in the interpreter,
but I didn't end up using them in this project. It is useful for things like WebGPU however. I ended up
here, just iterating over the values of the display as provided by the internal `BitArray` to draw rectangles
sized and translated to roughly center the image and large enough to play.

## Video

https://github.com/user-attachments/assets/c87c4ec8-c4e0-4533-aad0-ea6c89198b0f

## Acknowledgements

Bart Massey [mb2-tft-display](https://github.com/pdx-cs-rust-embedded/mb2-tft-display) skeleton code

## License

This project is licensed under the [MIT License][License].

[License]: ./LICENSE
[OXID8]: https://github.com/edibblepdx/Oxid-8
