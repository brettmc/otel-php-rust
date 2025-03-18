use cargo_metadata::MetadataCommand;
use std::fs;
use std::path::Path;

fn main() {
    #[cfg(target_os = "macos")]
    {
       println!("cargo:rustc-link-arg=-undefined");
       println!("cargo:rustc-link-arg=dynamic_lookup");
    }

    //get metadata about interesting dependencies
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to get cargo metadata");

    let package_names = ["opentelemetry", "phper", "tokio"];
    let mut versions = Vec::new();

    for &pkg in &package_names {
        let version = metadata
            .packages
            .iter()
            .find(|p| p.name == pkg)
            .map(|p| p.version.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        versions.push(format!(
            "pub const {}_VERSION: &str = \"{}\";",
            pkg.to_uppercase(),
            version
        ));
    }

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("package_versions.rs");
    fs::write(&dest_path, versions.join("\n"))
        .expect("Failed to write version file");
 }