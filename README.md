
# spaghetti os
My attempt at making a simple snake game at the kernel level

This has to be the worst code I have ever written, everything is unsafe and unreadable, but it works :)

![](https://i.imgur.com/vikkH0S.png)

## Compiling

1. Make sure you have the nightly toolchain installed
```sh
rustup toolchain install nightly
```
2. Make sure you have `xbuild`
```sh
cargo install cargo-xbuild
```
3. Make sure you have `rust-src`
```sh
rustup component add rust-src
```
4. Make sure you have `bootimage`
```sh
cargo install bootimage
```
5. Compile it
```sh
cargo bootimage --release
```

## Running
You can easily run it in the [QEMU](https://www.qemu.org/) virtual machine with the command below or you can burn the resulting `.bin` in `target/x86_64-unknown-none/release/`.
```
qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/release/bootimage-spaghetti-os.bin
```
