# LC3 VM (Rust)

A small LC-3 virtual machine written in Rust, based on:
https://www.jmeiners.com/lc3-vm/

## Build

```bash
cargo build
```

## Run an LC-3 program

```bash
cargo run -- path/to/program.obj
```

You can also pass multiple images:

```bash
cargo run -- image1.obj image2.obj
```

## Run lc3-rogue

This project includes a `Makefile` target that bootstraps tools, assembles `lc3-rogue`, and runs it:

```bash
make rogue-run
```

Controls in game: `w`, `a`, `s`, `d`.

## Test

```bash
cargo test
```
