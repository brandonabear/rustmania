# RustMania

`rustmania` is a simple command-line StepMania `.ssc` file parser.

> [!NOTE]
> This project is intended as a learning exercise to become familiar with coding in Rust.


## Quickstart

```shell
# Read and parse a `.ssc` simfile
cargo run -- --path path/to/file.ssc;

# Serialize as JSON
cargo run -- --path path/to/file.ssc -serialize;
```
