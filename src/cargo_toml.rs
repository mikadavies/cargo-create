pub(crate) const CARGO_TOML: &str = r#"[package]
name = "[NAME]"
version = "0.1.0"
edition = "2024"

[dependencies]
# ADD DEPENDENCIES HERE

[profile.dev]
lto = "off"
opt-level = 1

[profile.dev.package."*"]
opt-level = 3

[profile.release]
codegen-units = 1
lto = "fat"
panic = "abort"
strip = "symbols"
opt-level = 3
"#;
