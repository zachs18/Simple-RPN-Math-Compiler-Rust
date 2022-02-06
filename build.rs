fn main() -> Result<(), &'static str> {
    if std::env::var_os("CARGO_CFG_TARGET_ARCH") != Some("x86_64".into()) {
        return Err("This library currently only supports x86_64 linux.");
    }
    if std::env::var_os("CARGO_CFG_TARGET_OS") != Some("linux".into()) {
        return Err("This library currently only supports x86_64 linux.");
    }
    cc::Build::new()
        .file("src/code_segments.S")
        .compile("code_segments");
    Ok(())
}
