use copy_dir::copy_dir;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=trees");

    std::fs::remove_dir_all("build").unwrap_or_default();

    Command::new("forester")
        .args(["build", "trees", "--no-theme", "--no-assets"])
        .status()
        .expect("failed to run forester");

    Command::new("bun")
        .args([
            "build",
            "--minify",
            "--outdir=build",
            "--entry-naming",
            "[name].[hash].[ext]",
            "--asset-naming",
            "[name].[hash].[ext]",
            "./assets/scripts/index.js",
        ])
        .status()
        .expect("failed to run bun");

    std::fs::remove_file("build/index.css").unwrap_or_default();

    fn copy_files(dir: &str) {
        for entry in std::fs::read_dir(dir).expect("failed to read dir `public`") {
            let entry = entry.expect("failed to read entry");

            if entry.file_type().unwrap().is_dir() {
                if entry.file_name() == "fonts" {
                    println!("{:?}", entry);
                    copy_dir(entry.path(), Path::new("build/fonts"));
                }
                //copy_dir(entry, build).unwrap();
                //copy_dir(entry.path.to_str())
                copy_files(entry.path().to_str().unwrap());
            } else {
                let path = entry.path();
                let filename = path.file_name().unwrap().to_str().unwrap();
                let dest = format!("build/{}", filename);

                std::fs::copy(path, dest).expect("failed to copy file");
            }
        }
    }
    copy_files("public");
}
