#![allow(clippy::redundant_closure, clippy::redundant_pattern_matching)]

extern crate anyhow;
extern crate clap;
extern crate env_logger;
extern crate human_panic;
extern crate log;
extern crate wasm_pack;
extern crate which;

use anyhow::Result;
use clap::Parser;
use std::env;
use std::panic;
use wasm_pack::{command::run_wasm_pack, Cli, PBAR};

mod installer;

fn main() {
    env_logger::init();

    setup_panic_hooks();

    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        for cause in e.chain() {
            eprintln!("Caused by: {}", cause);
        }
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // Deprecate `init`
    if let Some("init") = env::args().nth(1).as_ref().map(|arg| arg.as_str()) {
        println!("wasm-pack init is deprecated, consider using wasm-pack build");
    }

    if let Ok(me) = env::current_exe() {
        // If we're actually running as the installer then execute our
        // self-installation, otherwise just continue as usual.
        if me
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("executable should have a filename")
            .starts_with("wasm-pack-init")
        {
            installer::install();
        }
    }

    let args = Cli::parse();

    PBAR.set_log_level(args.log_level);

    if args.quiet {
        PBAR.set_quiet(true);
    }

    run_wasm_pack(args.cmd)?;

    Ok(())
}

fn setup_panic_hooks() {
    let meta = human_panic::Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .authors(env!("CARGO_PKG_AUTHORS"))
        .homepage(env!("CARGO_PKG_HOMEPAGE"));

    let default_hook = panic::take_hook();

    if let Err(_) = env::var("RUST_BACKTRACE") {
        panic::set_hook(Box::new(move |info: &panic::PanicHookInfo| {
            // First call the default hook that prints to standard error.
            default_hook(info);

            // Then call human_panic.
            let file_path = human_panic::handle_dump(&meta, info);
            human_panic::print_msg(file_path, &meta)
                .expect("human-panic: printing error message to console failed");
        }));
    }
}
