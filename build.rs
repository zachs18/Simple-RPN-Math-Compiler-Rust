use std::io::{BufReader, BufRead, BufWriter, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("CARGO_CFG_TARGET_ARCH") != Some("x86_64".into()) {
        Err("This library currently only supports x86_64 linux.")?;
    }
    if std::env::var_os("CARGO_CFG_TARGET_OS") != Some("linux".into()) {
        Err("This library currently only supports x86_64 linux.")?;
    }

    let function_errors_file = BufReader::new(
        std::fs::File::open("function_errors.csv")?
    );
    let mut function_errors_asm = BufWriter::new(
        std::fs::File::create("src/function_errors.S")?
    );
    let mut function_errors_rs = BufWriter::new(
        std::fs::File::create("src/function/errors.rs")?
    );
    let mut function_error_from_raw = Vec::with_capacity(4096);
    let mut function_error_impl_display = Vec::with_capacity(4096);

    writeln!(function_errors_rs, "pub type FunctionErrorRaw = libc::intptr_t;")?;
    writeln!(function_errors_rs, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]")?;
    writeln!(function_errors_rs, "#[non_exhaustive]")?;
    writeln!(function_errors_rs, "#[repr(isize)] /* TODO: ensure isize == intptr_t */")?;
    writeln!(function_errors_rs, "pub enum FunctionError {{")?;

    writeln!(function_error_from_raw, "pub fn function_error_from_raw(raw: FunctionErrorRaw) -> Option<FunctionError> {{")?;
    writeln!(function_error_from_raw, "    if raw == 0 {{ None }} else {{ Some ( match raw {{")?;

    writeln!(function_error_impl_display, "impl std::fmt::Display for FunctionError {{")?;
    writeln!(function_error_impl_display, "    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{")?;
    writeln!(function_error_impl_display, "        use FunctionError::*;")?;
    writeln!(function_error_impl_display, "        let msg: &'static str = match self {{")?;

    for line in function_errors_file.lines() {
        let line = line?;
        let mut fields = line.split(',');
        let name: &str = fields.next().ok_or("too few fields")?;
        let value: std::num::NonZeroIsize = fields.next().ok_or("too few fields")?.parse()?;
        let msg: &str = fields.next().ok_or("too few fields")?;
        writeln!(function_errors_asm, "#define {} {}", name, value)?;

        writeln!(function_errors_rs, "    {} = {},", name, value)?;

        writeln!(function_error_from_raw, "        {} => FunctionError::{},", value, name)?;

        writeln!(function_error_impl_display, "            {} => {},", name, msg)?;
    }

    writeln!(function_errors_rs, "}}")?;

    writeln!(function_error_from_raw, "        _ => FunctionError::Other,")?;
    writeln!(function_error_from_raw, "    }} ) }}")?;
    writeln!(function_error_from_raw, "}}")?;

    writeln!(function_error_impl_display, "        }};")?;
    writeln!(function_error_impl_display, "        write!(fmt, \"{{}}\", msg)")?;
    writeln!(function_error_impl_display, "    }}")?;
    writeln!(function_error_impl_display, "}}")?;

    function_errors_rs.write(&*function_error_from_raw)?;
    function_errors_rs.write(&*function_error_impl_display)?;

    writeln!(function_errors_rs, "impl std::error::Error for FunctionError {{}}")?;

    function_errors_asm.flush()?;
    function_errors_rs.flush()?;

    cc::Build::new()
        .file("src/code_segments.S")
        .compile("code_segments");
    Ok(())
}
