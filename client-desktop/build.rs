fn main() {
    // Compile the minimal libvpx VP8 wrapper.
    let mut build = cc::Build::new();
    build.file("src/media/vpx_ffi/vpx_wrapper.c");
    build.include("/usr/include");
    build.flag_if_supported("-std=c99");
    build.compile("vpx_wrapper");

    // Link against the system libvpx discovered by pkg-config.
    let pkg = pkg_config::Config::new()
        .probe("vpx")
        .expect("libvpx not found");
    for lib in &pkg.libs {
        println!("cargo:rustc-link-lib={lib}");
    }
    for path in &pkg.link_paths {
        println!("cargo:rustc-link-search=native={}", path.display());
    }

    // Re-run build.rs if the wrapper changes.
    println!("cargo:rerun-if-changed=src/media/vpx_ffi/vpx_wrapper.c");
}

// Minimal pkg-config helper so we do not add a build-dependency on pkg-config.
mod pkg_config {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    pub struct Library {
        pub libs: Vec<String>,
        pub link_paths: Vec<PathBuf>,
    }

    pub struct Config;

    impl Config {
        pub fn new() -> Self {
            Self
        }

        pub fn probe(&self, name: &str) -> Option<Library> {
            let libs = Command::new("pkg-config")
                .args(["--libs", name])
                .output()
                .ok()
                .filter(|o| o.status.success())?;
            let libs = String::from_utf8_lossy(&libs.stdout);

            let mut found_libs = Vec::new();
            let mut link_paths = Vec::new();

            for token in libs.split_whitespace() {
                if let Some(lib) = token.strip_prefix("-l") {
                    found_libs.push(lib.to_string());
                } else if let Some(path) = token.strip_prefix("-L") {
                    link_paths.push(Path::new(path).to_path_buf());
                }
            }

            Some(Library {
                libs: found_libs,
                link_paths,
            })
        }
    }
}
