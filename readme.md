## Instructions

## Using hot reloading

1. Add `dylib` to the crate-type in list `loomz-client/Cargo.toml`
1. **IF** using release mode, comment out `panic = "abort"` in `./Cargo.toml`
2. Build client code with the hot-reload feature using `cargo build -p loomz-client --features hot-reload`
3. Run the main app with the hot-reloading feature: `cargo run -p loomz --features hot-reload`
4. Rebuild the client code with the command from step 3 when needed
5. You will see the message `CLIENT RELOADED` in the main app console when the new library was reloaded
