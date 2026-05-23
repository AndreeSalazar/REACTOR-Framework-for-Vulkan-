// =============================================================================
// REACTOR CLI Tool
// =============================================================================
// Provides command line utilities for building, cooking, and packaging assets.
// Usage: cargo run --bin reactor cook --input assets/ --output cooked/
// =============================================================================

use std::env;
use reactor_vulkan::resources::AssetCooker;

fn print_help() {
    println!("⚛ REACTOR Framework CLI");
    println!("Usage:");
    println!("  reactor cook [options]");
    println!();
    println!("Options:");
    println!("  -i, --input <dir>   Input raw assets directory (default: assets)");
    println!("  -o, --output <dir>  Output cooked assets directory (default: cooked)");
    println!("  -h, --help          Print this help message");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "cook" => {
            let mut input = "assets".to_string();
            let mut output = "cooked".to_string();

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "-i" | "--input" => {
                        if i + 1 < args.len() {
                            input = args[i + 1].clone();
                            i += 2;
                        } else {
                            eprintln!("Error: Missing value for --input");
                            std::process::exit(1);
                        }
                    }
                    "-o" | "--output" => {
                        if i + 1 < args.len() {
                            output = args[i + 1].clone();
                            i += 2;
                        } else {
                            eprintln!("Error: Missing value for --output");
                            std::process::exit(1);
                        }
                    }
                    "-h" | "--help" => {
                        print_help();
                        return;
                    }
                    other => {
                        eprintln!("Error: Unknown option '{}'", other);
                        print_help();
                        std::process::exit(1);
                    }
                }
            }

            println!("🍳 Starting REACTOR Asset Cooker...");
            println!("👉 Input directory:  {}", input);
            println!("👉 Output directory: {}", output);

            let mut cooker = match AssetCooker::new(&input, &output) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("❌ Failed to initialize AssetCooker: {}", e);
                    std::process::exit(1);
                }
            };

            match cooker.cook_all() {
                Ok(_) => {
                    println!("✅ Cooking completed successfully!");
                }
                Err(e) => {
                    eprintln!("❌ Error during cooking: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "help" | "-h" | "--help" => {
            print_help();
        }
        other => {
            eprintln!("Error: Unknown command '{}'", other);
            print_help();
            std::process::exit(1);
        }
    }
}
