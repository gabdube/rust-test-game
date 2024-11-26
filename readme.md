## Instructions

## Using hot reloading

1. **IF** using release mode, comment out `panic = "abort"` in `./Cargo.toml`
2. Build client code with the hot-reload feature using `cargo build -p loomz-client --features hot-reload`
3. Run the main app with the hot-reloading feature: `cargo run -p loomz --features hot-reload`
4. Rebuild the client code with the command from step 2 when needed
5. You will see the message `CLIENT RELOADED` in the main app when the new library was reloaded
