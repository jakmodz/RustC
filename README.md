# C compiler in rust

C compiler is a simple C compiler written in Rust. It is designed to be easy to understand and modify, making it a great starting point for anyone interested in learning about compiler construction.
Following Nora Sandlers book "Build Your Own C Compiler". Planed to Extend it with more features in the future.

## Requirements
- Rust (latest stable version)
- Cargo (comes with Rust)
- GCC for linking

## Building
```bash
cargo build --release
```

## Usage
```bash
./target/release/Compiler <path-to-c-file>
```
## Future Features
- WebAssembly Target
- Arm Target
- More C language features
- Better error handling and reporting