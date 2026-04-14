use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let board_memory = select_memory_script();

    fs::copy(&board_memory, out.join("memory.x")).unwrap();

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed={}", board_memory);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/arch/cortex_m/boot.S");
    println!("cargo:rerun-if-changed=src/arch/cortex_m/pendsv.S");

    cc::Build::new()
        .file("src/arch/cortex_m/boot.S")
        .file("src/arch/cortex_m/pendsv.S")
        .compile("cortexos_asm");
}

fn select_memory_script() -> String {
    let mut boards: Vec<String> = env::vars_os()
        .filter_map(|(key, _)| {
            let key = key.to_string_lossy();
            let suffix = key.strip_prefix("CARGO_FEATURE_BOARD_")?;
            Some(suffix.to_ascii_lowercase().replace('_', "-"))
        })
        .collect();

    boards.sort();
    boards.dedup();

    let board = match boards.as_slice() {
        [] => panic!("no board feature enabled"),
        [single] => single,
        many => panic!("multiple board features enabled: {}", many.join(", ")),
    };

    let memory_script = format!("memory/{board}.x");
    if !Path::new(&memory_script).exists() {
        panic!(
            "missing memory script for board feature '{board}': expected {}",
            memory_script
        );
    }

    memory_script
}
