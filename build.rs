use std::io::Result;
fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=src/messages.proto");
    prost_build::compile_protos(&["src/messages.proto"], &["src/"])?;
    Ok(())
}
