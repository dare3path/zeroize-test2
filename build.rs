use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use anyhow::Context;

/// We define this cfg so you can do this in main.rs:
///        #[cfg(not(rustls_pki_types_zeroize))]
///        compile_error!("build.rs didn't run");
/// because if it does run, it'll either panic or define that.
const CFG_RUSTLS_PKI_TYPES_ZEROIZE:&str="rustls_pki_types_zeroize";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Rerun if dependencies change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");//redundant
    //if Cargo.lock is manually changed, this next line regens it, losing the changes!
    //because even if doing touch build.rs would regen it!
    println!("cargo:rerun-if-changed=Cargo.lock");//overkill? keeps it in sync with Cargo.toml
    println!("cargo::rustc-check-cfg=cfg({})", CFG_RUSTLS_PKI_TYPES_ZEROIZE);

    // XXX: can see these eprintln(s) if `cargo build -vv`

    // Log all environment variables
    eprintln!("Environment variables:");
    for (key, value) in env::vars() {
        eprintln!("{}={}", key, value);
    }

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let temp_dir = Path::new(&out_dir).join("test_zeroize_crate");
    let test_file = temp_dir.join("src").join("main.rs");
    let cargo_toml = temp_dir.join("Cargo.toml");
    let cargo_lock = temp_dir.join("Cargo.lock");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    // Create temporary crate directory
    fs::create_dir_all(temp_dir.join("src")).expect("Failed to create temp crate dir");

    // Write test program
    let test_code = r#"
        #[allow(dead_code)]
        const fn assert_rustls_pki_types_zeroize<T: zeroize::Zeroize + zeroize::ZeroizeOnDrop>() {}
        const _: () = {
            assert_rustls_pki_types_zeroize::<rustls_pki_types::PrivatePkcs8KeyDer<'static>>();
            assert_rustls_pki_types_zeroize::<rustls_pki_types::PrivateKeyDer<'static>>();
        };

        fn main() {}
    "#;
    File::create(&test_file)
        .context(format!("Failed to create file {:?}", test_file))?
        .write_all(test_code.as_bytes())
        .context(format!("Failed to write file {:?}", test_file))?;

    // Copy Cargo.toml
    let src_cargo_toml = Path::new(&manifest_dir).join("Cargo.toml");
    fs::copy(&src_cargo_toml, &cargo_toml).expect("Failed to copy Cargo.toml");

    // Copy Cargo.lock it should exist!
    let src_cargo_lock = Path::new(&manifest_dir).join("Cargo.lock");
    if !src_cargo_lock.exists() {
        panic!("Cargo.lock not found at {}; should be impossible.", src_cargo_lock.display());
    }
    fs::copy(&src_cargo_lock, &cargo_lock).expect("Failed to copy Cargo.lock");

    // Run cargo build in temp_dir with safety check
    let temp_dir_str = temp_dir.to_str().expect("Invalid temp_dir path");
    if temp_dir_str == manifest_dir {
        panic!("Error: temp_dir is the same as CARGO_MANIFEST_DIR(={}), aborting to prevent recursion", manifest_dir);
    }

    // redundant now, and possibly dangerous?(if other things assume we're in our proj. dir, below)
    // but anyway, just making doubly sure!
    if let Err(e) = env::set_current_dir(&temp_dir) {
        panic!("Failed to set working directory to {}: {}", temp_dir_str, e);
    }

    // Use CARGO env var
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(&cargo);
    cmd
        .arg("build")
        .arg("--color=never")
        .arg("--manifest-path")
        .arg(&cargo_toml)
        .arg("--target-dir")
        .arg(temp_dir.join("target"))
        //      --locked                Assert that `Cargo.lock` will remain unchanged
        //      --offline               Run without accessing the network
        //      --frozen                Equivalent to specifying both --locked and --offline
        .arg("--offline")
        .arg("--locked")
        .current_dir(&temp_dir);

    let cmd_str=format!("{} {:?}", cmd.get_program().to_string_lossy(), cmd.get_args().collect::<Vec<_>>());
    let cwd=std::env::current_dir().expect("Failed to get CWD");
    if cwd != temp_dir {
        panic!("cwd isn't same as expected, is: {:?}, expected:{:?}", cwd, temp_dir);
    }
    eprintln!("Current working dir is: {}", cwd.display());
    eprintln!("Running cargo command: {}", cmd_str);
    let output=cmd
        .output()
        .context(format!("Failed to run {}",cmd_str))?;

    // Log the command and its output
    eprintln!("cargo stdout: {}", String::from_utf8_lossy(&output.stdout));
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("cargo stderr: {}", stderr);

    // Set cfg based on compilation result
    if output.status.success() {
        eprintln!("cargo build succeeded, rustls-pki-types crate supports Zeroize");
        eprintln!("cargo build succeeded, setting cfg {}", CFG_RUSTLS_PKI_TYPES_ZEROIZE);
        println!("cargo:rustc-cfg={}", CFG_RUSTLS_PKI_TYPES_ZEROIZE);
    } else {
        if stderr.contains ("error[E0277]: the trait bound `")
           && (stderr.contains(": Zeroize` is not satisfied") 
                || stderr.contains(": ZeroizeOnDrop` is not satisfied"))
           && (stderr.contains("the trait `DefaultIsZeroes` is not implemented for `")
                || stderr.contains("the trait `ZeroizeOnDrop` is not implemented for `"))
        {
             //eprintln!("cargo build failed with the expected Zeroize error");
               panic!(
                   "{}",
"!!!!!!!!!!!
The crate 'rustls-pki-types' does not implement zeroize::Zeroize and/or zeroize::ZeroizeOnDrop for things like PrivatePkcs8KeyDer<'static>.
Try bumping to version >= 1.12.0 or use this:
[patch.crates-io]
rustls-pki-types = { git = \"https://github.com/rustls/pki-types.git\", rev = \"b59e08d49911b10c423d25bd9040cfbe5a6042ff\" }
Actually that doesn't have ZeroizeOnDrop, so try patching the cloned pki-types repo with this patch:
./zeroize_on_drop.patch
then set that path to it in Cargo.toml 's [patch.crates-io] section.
!!!!!!!!!!!"
             );
         } else {
             panic!("!!!!!!!! 'cargo build', from within build.rs, failed unexpectedly as seen above.");
         }
    }
    //panic!("foo");
    Ok(())
}
