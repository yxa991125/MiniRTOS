use std::{env, fs, path::PathBuf};

fn main() {
    // ---------- 让 cortex-m-rt 链接时能找到 memory.x ----------
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    // 把根目录的 memory.x 复制到 OUT_DIR
    fs::copy("memory.x", out.join("memory.x")).unwrap();

    // 告诉链接器去 OUT_DIR 找链接脚本（memory.x 会被 cortex-m-rt 的 link.x 使用）
    println!("cargo:rustc-link-search={}", out.display());

    // 变更检测
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/arch/cortex_m/boot.S");
    println!("cargo:rerun-if-changed=src/arch/cortex_m/pendsv.S");

    // ---------- 编译汇编 ----------
    cc::Build::new()
        .file("src/arch/cortex_m/boot.S")
        .file("src/arch/cortex_m/pendsv.S")
        .compile("cortexos_asm");
}
