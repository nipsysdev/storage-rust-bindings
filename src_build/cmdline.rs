/// Compiles the cmdline_symbols.c file to provide missing Nim symbols
pub fn compile_cmdline_symbols() {
    cc::Build::new()
        .file("src_build/cmdline_symbols.c")
        .compile("cmdline_symbols");
}
