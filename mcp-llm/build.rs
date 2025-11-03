use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=include/engine.h");

    // Generate LiteRT-LM bindings from C API
    let bindings = bindgen::Builder::default()
        .header("include/engine.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Allowlist the LiteRT-LM C API functions
        .allowlist_function("litert_lm_.*")
        .allowlist_type("LiteRtLm.*")
        .allowlist_type("InputData.*")
        .allowlist_var("kInput.*")
        // Generate only C bindings (no C++)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Link against LiteRT-LM libraries in dependency order
    let lib_path = format!("{}/lib", env::var("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:rustc-link-search=native={}", lib_path);
    
    // Core runtime libraries (dependencies first)
    println!("cargo:rustc-link-lib=static=litert_lm_lib");
    println!("cargo:rustc-link-lib=static=engine_impl");
    println!("cargo:rustc-link-lib=static=session_basic");
    println!("cargo:rustc-link-lib=static=conversation");
    println!("cargo:rustc-link-lib=static=engine_settings");
    println!("cargo:rustc-link-lib=static=io_types");
    println!("cargo:rustc-link-lib=static=llm_executor_settings");
    println!("cargo:rustc-link-lib=static=llm_executor_io_types");
    
    // C API library (last)
    println!("cargo:rustc-link-lib=static=engine");

    // Link system libraries
    println!("cargo:rustc-link-lib=dylib=c++");
    println!("cargo:rustc-link-lib=dylib=System");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
}