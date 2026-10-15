mod app;
mod gui;
mod input;
mod terminal;

use std::{env, error::Error};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BackendChoice {
    Gui,
    Terminal,
}

fn main() -> Result<(), Box<dyn Error>> {
    let backend = parse_backend(env::args().skip(1))?;
    match backend {
        BackendChoice::Gui => gui::run(),
        BackendChoice::Terminal => terminal::run(),
    }
}

fn parse_backend(args: impl IntoIterator<Item = String>) -> Result<BackendChoice, Box<dyn Error>> {
    let mut args = args.into_iter();
    let Some(flag) = args.next() else {
        return Ok(BackendChoice::Gui);
    };
    if flag == "--help" || flag == "-h" {
        println!("crosshair [--backend gui|terminal]\n\nDefaults to the native GUI backend.");
        std::process::exit(0);
    }
    if flag != "--backend" {
        return Err(
            format!("unknown argument: {flag}\nusage: crosshair [--backend gui|terminal]").into(),
        );
    }
    match args.next().as_deref() {
        Some("gui") => Ok(BackendChoice::Gui),
        Some("terminal") => Ok(BackendChoice::Terminal),
        Some(other) => Err(format!("unknown backend: {other}; expected gui or terminal").into()),
        None => Err("--backend requires gui or terminal".into()),
    }
}
