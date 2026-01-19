/// Compiles the cmdline_symbols.c file to provide missing Nim symbols
pub fn compile_cmdline_symbols() {
    println!("  [CMDLINE] Starting compile_cmdline_symbols");

    let source_file = "src_build/cmdline_symbols.c";
    println!("  [CMDLINE] Source file: {}", source_file);

    if std::path::Path::new(source_file).exists() {
        println!("  [CMDLINE] ✓ Source file exists");
    } else {
        println!("  [CMDLINE] ✗ Source file does not exist!");
    }

    println!("  [CMDLINE] Compiling with cc crate...");
    cc::Build::new()
        .file(source_file)
        .compile("cmdline_symbols");

    println!("  [CMDLINE] ✓ Compilation completed successfully");
    println!("  [CMDLINE] ✓ Output library: cmdline_symbols");
}
