# Development

## Prerequisites

- GTK4 4.20+
- libadwaita 1.8+
- Rust 1.83+
- asusctl installed and configured

## Setup

Copy the GSettings schema to your local schemas directory and compile:

```bash
cp resources/com.github.bl4ckspell7.asusctl-gui.gschema.xml ~/.local/share/glib-2.0/schemas/

glib-compile-schemas ~/.local/share/glib-2.0/schemas/
```

## Run

```bash
cargo run
```

## Build

```bash
cargo build
```

## Test

Run tests:

```bash
cargo test
```

### Coverage

Install cargo-llvm-cov:

```bash
cargo install cargo-llvm-cov
```

Generate HTML coverage report:

```bash
cargo llvm-cov --html
```

The report will be generated in `target/llvm-cov/html/index.html`.

Generate text summary:

```bash
cargo llvm-cov
```
