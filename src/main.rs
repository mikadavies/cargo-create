use std::{
    ffi::OsStr,
    fs::File,
    io::{IsTerminal, Write},
    path::Path,
};

use termcolor::{ColorSpec, StandardStream, WriteColor};

mod cargo_config;
mod cargo_toml;

type Stdout<'a> = &'a mut StandardStream;

fn main() {
    let args: Vec<String> = read_command_line_args();

    let stdout: Stdout = &mut StandardStream::stdout(match std::io::stdin().is_terminal() {
        true => termcolor::ColorChoice::Always,
        false => termcolor::ColorChoice::Never,
    });

    if args.is_empty() {
        warn_empty_args(stdout);
    }
    // We can now use unchecked access to the first element of the array, as `len > 0`
    else if args.contains(&"--help".to_string()) {
        send_help();
    } else if args.len() == 1 {
        let name: &str = unsafe { args.get_unchecked(0) };
        let success: Result<_, _> = create_default_project(name, true, true, false);
        handle_output(success, stdout, name);
    } else {
        let inputs: Inputs = parse_args(&args, stdout);
        let success: Result<_, _> = create_from_inputs(inputs);
        handle_output(success, stdout, unsafe { args.get_unchecked(0) });
    }
}

// === DEFAULT CONFIG ===
fn create_default_project(
    name: &str,
    config: bool,
    opt: bool,
    lib: bool,
) -> Result<(), std::io::Error> {
    // Create path
    let path_str: &str = &format!("./{name}");
    let path: &Path = Path::new(path_str);

    // Make directories for new project
    std::fs::create_dir_all(path.join("src"))?;

    // Make files from default template
    if config && opt {
        // Create `.cargo/config.toml`
        std::fs::create_dir(path.join(".cargo"))?;
        let mut config: File = File::create_new(path.join(".cargo/config.toml"))?;
        config.write_all(cargo_config::CONFIG)?;
    }
    // Make `src/main.rs`
    {
        let mut mainf: File = File::create_new(match lib {
            false => path.join("src/main.rs"),
            true => path.join("src/lib.rs"),
        })?;
        mainf.write_all(b"fn main(){}")?;
    }
    // Make 'Cargo.toml`
    {
        let mut cargo: File = File::create_new(path.join("Cargo.toml"))?;

        let contents: String = match opt {
            true => cargo_toml::CARGO_TOML.replacen("[NAME]", name, 1),
            false => {
                let mut lines: Vec<&str> = cargo_toml::CARGO_TOML.lines().collect();
                let name_line: &str = &format!("name = \"{name}\"");
                unsafe { *lines.get_unchecked_mut(1) = name_line };
                lines[..6]
                    .iter()
                    .map(|line| [*line, "\n"].concat())
                    .collect::<Vec<String>>()
                    .concat()
            }
        };
        cargo.write_all(contents.as_bytes())?;
    }
    Ok(())
}

// === SPECIAL CONFIG ===
struct Inputs<'a> {
    config: bool,
    lib: bool,
    opt: bool,
    template_dir: Option<&'a Path>,
    name: &'a str,
}
impl Default for Inputs<'_> {
    fn default() -> Self {
        Self {
            config: true,
            lib: false,
            opt: true,
            template_dir: None,
            name: "",
        }
    }
}

fn parse_args<'a>(args: &'a [String], stdout: Stdout) -> Inputs<'a> {
    let mut inputs: Inputs = Inputs {
        name: unsafe { args.get_unchecked(0) },
        ..Default::default()
    };
    args.iter().skip(1).for_each(|arg| {
        // Set template directory for project, if specified
        if arg.contains("--template-dir") {
            let template_path: Option<&str> = arg.split('=').nth(1);
            if let Some(path_str) = template_path {
                inputs.template_dir = Some(Path::new(path_str));
            } else {
                warn(stdout, "Incorrectly formatted argument: \"template-dir\"");
            }
        }
        // Use `lib.rs` instead of `main.rs`
        else if arg == "--lib-not-main" {
            inputs.lib = true;
        }
        // Disable optimisations
        else if arg == "--no-opt" {
            inputs.opt = false;
        }
        // Disable `config.toml`
        else if arg == "--no-config" {
            inputs.config = false;
        }
        // Otherwise it is an unsupported argument. Ignore.
        else {
            warn(stdout, &format!("Unknown argument: \"{arg}\". Ignoring."));
        }
    });

    inputs
}

fn create_from_inputs(inputs: Inputs) -> Result<(), std::io::Error> {
    if let Some(template_dir) = inputs.template_dir {
        // copy folders and files from template path
        let path_str: &str = &format!("./{:}", inputs.name);
        let out_path: &Path = Path::new(path_str);
        std::fs::create_dir(out_path)?;
        copy_dir_recursive(template_dir, out_path)?;
        // Update project name in `Cargo.toml`
        let cargo_toml_path: &Path = &out_path.join("Cargo.toml");
        let cargo_contents: String =
            std::fs::read_to_string(cargo_toml_path)?.replacen("[NAME]", inputs.name, 1);
        std::fs::write(cargo_toml_path, cargo_contents)
    } else {
        create_default_project(inputs.name, inputs.config, inputs.opt, inputs.lib)
    }
}

fn copy_dir_recursive(input: &Path, output: &Path) -> Result<(), std::io::Error> {
    for entry in input.read_dir()? {
        let path: &Path = &entry?.path();
        // If the current entry is a directory, make the new directory in the new project, then create all children entries recursively
        if path.is_dir() {
            let dir_name: &OsStr = path
                .file_name()
                .ok_or(std::io::Error::other("Directory has no name!"))?;
            let output_dir: &Path = &output.join(dir_name);
            std::fs::create_dir(output_dir)?;
            copy_dir_recursive(path, output_dir)?;
        } else {
            std::fs::copy(
                path,
                output.join(
                    path.file_name()
                        .ok_or(std::io::Error::other("File has no name!"))?,
                ),
            )?;
        }
    }

    Ok(())
}

// === HELP (Documentation) ===
fn send_help() {
    println!(
        r#"`cargo-create` is a tool to create rust projects with a given configuration.
--
The first argument must always be the name of your new project. This will also be the project directory name.

If used as is, this will generate a default project with a `main.rs` file, a `Cargo.toml` file with useful optimisations enabled, and a `.cargo/config.toml` file with more compiler optimisations enabled. These features can be toggled using the following arguments:

    --no-opt       : disables all optimisation options
    --no-config    : omits the `.cargo` directory, therefore doesn't create `config.toml`
    --lib-not-main : replaces `main.rs` with `lib.rs`

Additionally, you can specify a custom project template with the `--template-dir` argument. The correct format is:
    --template-dir=path/to/directory
Adding a space will not result in this setting being ignored. Paths can be relative or absolute.

If a custom template is specified, other arguments are ignored.
"#
    );
}

// === CLI utilities ===

fn read_command_line_args() -> Vec<String> {
    std::env::args().skip(1).collect()
}

fn warn_empty_args(stdout: Stdout) {
    make_error_colour(stdout);
    let _ = writeln!(
        stdout,
        "No arguments were provided. At least the name of the project is required."
    );
    reset_colour(stdout);
}

fn handle_output(output: Result<(), std::io::Error>, stdout: Stdout, name: &str) {
    if let Err(e) = output {
        make_error_colour(stdout);
        let _ = writeln!(stdout, "Failed to create project. Reason:");
        let _ = writeln!(stdout, "{e}");
        reset_colour(stdout);
    } else {
        make_success_colour(stdout);
        let _ = writeln!(stdout, "Created project successfully! Name: {name}");
        reset_colour(stdout);
    }
}

fn make_error_colour(stdout: Stdout) {
    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Red)));
}
fn make_success_colour(stdout: Stdout) {
    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Green)));
}
fn reset_colour(stdout: Stdout) {
    let _ = stdout.set_color(&ColorSpec::default());
}
fn warn(stdout: Stdout, msg: &str) {
    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)));
    let _ = writeln!(stdout, "{msg}");
    reset_colour(stdout);
}
