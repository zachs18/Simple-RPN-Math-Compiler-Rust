use std::io::{BufReader, BufRead, BufWriter, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut temp = std::fs::File::create("/tmp/aaa.txt")?;
    for (var, val) in std::env::vars_os() {
        writeln!(temp, "{:?}: {:?}", var, val)?;
    }
    drop(temp);

    let target = std::env::var_os("TARGET").ok_or("Invalid target")?;
    let mut code_segments_path = std::path::Path::new("src")
        .join("code_segments");
    code_segments_path.push(target);
    code_segments_path.set_extension("S");

    let out_dir = std::env::var_os("OUT_DIR").ok_or("OUT_DIR not set")?;
    let out_dir = std::path::Path::new(&out_dir);

    let function_errors_file = BufReader::new(
        std::fs::File::open("function_errors.csv")?
    );
    let function_errors_asm_path = out_dir.join("function_errors.S");
    let mut function_errors_asm = BufWriter::new(
        std::fs::File::create(&function_errors_asm_path)?
    );
    let function_errors_rs_path = out_dir.join("function_errors.rs");
    let mut function_errors_rs = BufWriter::new(
        std::fs::File::create(&function_errors_rs_path)?
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
        .include(out_dir)
        .file(code_segments_path)
        .compile("code_segments");
    Ok(())
}
