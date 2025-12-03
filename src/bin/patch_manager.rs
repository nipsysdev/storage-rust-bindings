//! Patch Manager CLI
//!
//! A command-line tool for managing Android patches in the patch system.

use clap::{Arg, Command};
use codex_bindings::patch_system::{get_android_arch_from_target, PatchEngine};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("patch_manager")
        .version("1.0.0")
        .about("Android Patch Manager")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Verbose output")
                .action(clap::ArgAction::SetTrue)
                .global(true),
        )
        .subcommand(
            Command::new("apply")
                .about("Apply patches for an architecture")
                .arg(
                    Arg::new("arch")
                        .help("Architecture to apply patches for (arm64, x86_64, arm32, x86)")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("validate")
                .about("Validate patches for an architecture")
                .arg(
                    Arg::new("arch")
                        .help("Architecture to validate patches for")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("List available patches for an architecture")
                .arg(Arg::new("arch").help("Architecture to list patches for")),
        )
        .subcommand(Command::new("info").about("Show patch system information"))
        .subcommand(
            Command::new("auto")
                .about("Auto-detect architecture from TARGET env var and apply patches"),
        )
        .get_matches();

    // Create patch engine
    let verbose = matches.get_flag("verbose");

    let engine = PatchEngine::new(verbose)?;

    match matches.subcommand() {
        Some(("apply", sub_matches)) => {
            let arch = sub_matches.get_one::<String>("arch").unwrap();
            println!("Applying patches for architecture: {}", arch);
            let applied_patches = engine.apply_patches_for_arch(arch)?;
            println!("Successfully applied {} patches:", applied_patches.len());
            for patch in applied_patches {
                println!("  - {}", patch);
            }
        }
        Some(("validate", sub_matches)) => {
            let arch = sub_matches.get_one::<String>("arch").unwrap();
            println!("Validating patches for architecture: {}", arch);
            engine.validate_patches_for_arch(arch)?;
            println!("All patches are correctly applied!");
        }
        Some(("list", sub_matches)) => {
            if let Some(arch) = sub_matches.get_one::<String>("arch") {
                let patches = engine.get_patches_for_arch(arch)?;
                println!("Patches for architecture {}:", arch);
                for (i, patch) in patches.iter().enumerate() {
                    println!("  {}. {}", i + 1, patch);
                }
            } else {
                let archs = engine.get_available_architectures()?;
                println!("Available architectures:");
                for arch in archs {
                    println!("  - {}", arch);
                }
            }
        }
        Some(("info", _)) => {
            let info = codex_bindings::build_integration::get_patch_system_info()?;
            println!("{}", info);
        }
        Some(("auto", _)) => {
            let target = env::var("TARGET").unwrap_or_default();
            if !target.contains("android") {
                eprintln!("Not an Android target. TARGET={}", target);
                eprintln!("This command only works for Android builds.");
                std::process::exit(1);
            }

            let arch = get_android_arch_from_target(&target).ok_or("Unsupported Android target")?;

            println!(
                "Auto-detected architecture: {} from target: {}",
                arch, target
            );
            let applied_patches = engine.apply_patches_for_arch(arch)?;
            println!("Successfully applied {} patches:", applied_patches.len());
            for patch in applied_patches {
                println!("  - {}", patch);
            }
        }
        _ => {
            eprintln!("No subcommand provided. Use --help for usage information.");
            std::process::exit(1);
        }
    }

    Ok(())
}
