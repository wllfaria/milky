# Milky

**Milky** is a UCI chess engine written in Rust.

## Strength

Milky plays at an estimated **~2000 Elo** level based on informal testing
against other engines. This is a rough approximation and not a result of
formal automated testing.

Want to help improve this estimate? Run Milky in engine tournaments and
share the results!

## Compiling Milky

All you need is cargo installed, and you can build it from the command line.

```bash
cargo build

# or, for release builds.
cargo build --release

# If you simply want to run milky.
cargo run
```

Running `cargo run` will start the UCI loop.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## License

Milky and all of its sub-projects are licensed under the MIT license.
