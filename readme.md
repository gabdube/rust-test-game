## Instructions

Rust test game (codename Loomz), is a game / game-engine built from scratch, with minimal dependencies, using the Rust programming language and Vulkan for the graphics. The goal of this project is to create a light, powerful, and reusable template for game development.

The project has been tested on Windows and Linux. MacOs will soon be supported as soon as I figure what's wrong with MoltenVk on my setup.

This project is under heavy development, and currently only display a tiny guy to following your cursor with some basic GUI.

There will be a article that will go over the architecture at some point in the future. 

# Project structure

* **assets**: All the game assets. Note that the git project only includes shader sources and the asset bundle definition (assets.csv)
* **assets/dev**: Development assets
* **loomz**: Base project. Combines the client and the engine together
* **loomz-client**: Game client code & systems
* **loomz-engine**: Rendering code & systems for this project
* **loomz-engine-core**: Reusable rendering code and system
* **loomz-shared**: Add shared data between loomz, loomz-client, and loomz-engine
* **loomz-tools**: Tooling to process the dev assets
* **vk**: Custom, very unsafe, vulkan wrapper

## Running instructions

You need rust and a system that supports Vulkan 1.2

After cloning the repo and downloading the assets from the release section ( https://github.com/gabdube/rust-test-game/releases/tag/0.0.1assets ), just run this command from your console

```rust
cargo run -p loomz --release
```

Running the program without `release` will enable the Vulkan validation layers.

## Using multithreading

With this option, `loomz`, `loomz-client`, and `loomz-engine` will run on different threads. Without it, they will all run on one thread. Multithreading works will all other
features. 

```rust
cargo run -p loomz  --features multithreading
```

## Using assets reloading

You can enable assets reloading at runtime. Only shaders can be reloaded for now

```rust
cargo run -p loomz  --features reload-assets
```

## Using hot reloading

This project support hot reloading for the game client code (aka the `loomz-client`). However, you need to configure a few things before enabling the feature:

1. Add `dylib` to the crate-type in list `loomz-client/Cargo.toml`
1. **When** using release mode, comment out `panic = "abort"` in `./Cargo.toml`
2. Build client code with the hot-reload feature using `cargo build -p loomz-client --features hot-reload`
3. Run the main app with the hot-reloading feature: `cargo run -p loomz --features hot-reload`
4. Rebuild the client code with the command from step 3 when needed
5. You will see the message `CLIENT RELOADED` in the main app console when the new library is reloaded

Command: 
```rust
cargo run -p loomz  --features hot-reload
```


## Tooling

Most assets needs to be preprocessed before being usable in this project. This is done using 3rd party sofware and the `loomz-tool`

Dependencies:

* Compressonator ( https://gpuopen.com/compressonator/ ): Used to compress textures
* Msdf-atlas-gen ( https://github.com/Chlumsky/msdf-atlas-gen ): Used to generate the fonts atlas
* Lunarg Vulkan SDK ( https://www.lunarg.com/vulkan-sdk/ ): To compile the shaders

To run the tools use this command:

```
cargo run -p loomz-tools --release -- *options*
```

where options can be:

* `-f string`. Only process assets with `string` in their path name
* `-c command_name`. Execute the script named `command_name`. Possible values are: `remove_ds_store`, `generate_sprites`, `generate_fonts`
* Without any arguments, the tools will compile shaders and compress all textures

## Credits

* This project use the Tiny Sword asset pack by PixelFrog ( https://pixelfrog-assets.itch.io/tiny-swords )
