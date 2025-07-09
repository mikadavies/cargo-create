# cargo-create
A tool to quickly generate optimised or custom-structured Rust projects.

## What and why
This project was primarily created for personal use, as I often apply the same optimisations to most projects through `Cargo.toml` and `.cargo/config.toml`.
This tool is simply a Rust-centric method for achieving this.

The default behaviour takes a project name as its only input argument, and produces a Rust project of that name in a child directory of the current location. 
The default configuration should be suitable for most projects, with a few caveats:
- `config.toml` uses nightly-specific options.
- The `-Z threads` rustflag is set to 8 by default, this might exceed the capacity of some devices.
- The default Linux linker is set to the [mold linker](https://github.com/rui314/mold)

To provide some extra flexibility, it comes with a few options:
- `--lib-not-main` replaces the `main.rs` file with a `lib.rs` file
- `--no-opt` disables all optimisations. This is effectively equivalent to `cargo new`
- `--no-config` disables the creation of `./cargo/config.toml`

Finally, cargo-create supports custom project templates. These can be supplied through the `--template-dir` flag. To accurately name the project in `Cargo.toml`, please ensure that the name field in the template file is set to: `name = "[NAME]"`. 
If a custom template is provided, the other arguments are ignored.

### Example usage:
```
$ cargo-create myproject
> Created project successfully! Name: myproject
```
```
$ cargo-create myproject --lib-not-main
> Created project successfully! Name: myproject
```
```
$ cargo-create myproject --template-dir=path/to/template
> Created project successfully! Name: myproject
```
It is worth noting that the tool currently does not support spaces when specifying the template directory. Therefore, `--template-dir = path/to/template` and `--template-dir=path/to/multiple words/template` will fail to recognise the argument, and create a default configuration. If the template path contains spaces, please pass it as a string: 
```
$ cargo-create myproject --template-dir="path/to/multiple words/template"
> Created project successfully! Name: myproject
```
