// - Configures protobuf compilation
// - Generates Rust code from proto definitions

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell cargo to recompile if proto files change
    println!("cargo:rerun-if-changed=proto/service.proto");
    println!("cargo:rerun-if-changed=proto/common.proto");
    println!("cargo:rerun-if-changed=proto/trace.proto");
    println!("cargo:rerun-if-changed=proto/resource.proto");
    
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .out_dir("src/proto")
        .compile(
            &["proto/service.proto"],
            &["proto"],
        )?;

    Ok(())
}
