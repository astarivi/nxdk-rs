// SPDX-License-Identifier: MIT
extern crate bindgen;

use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::Write;

fn link_lib(lib: &str) {
    println!("cargo:rustc-link-lib={}", lib);
}

fn gen_bindings(nxdk_dir: &str, lib_path: &str, header: &str) {
    let bindings = bindgen::builder()
        .header(format!("{}/lib/{}/{}.h", nxdk_dir, lib_path, header))
        .clang_arg(format!("-I{}/lib", nxdk_dir))
        .clang_arg(format!("-I{}/lib/xboxrt/libc_extensions", nxdk_dir))
        .clang_arg(format!("-I{}/lib/pdclib/include", nxdk_dir))
        .clang_arg(format!("-I{}/lib/pdclib/platform/xbox/include", nxdk_dir))
        .clang_arg(format!("-I{}/lib/usb/libusbohci/inc", nxdk_dir))
        .clang_arg(format!("-I{}/lib/usb/libusbohci_xbox/", nxdk_dir))
        .clang_arg(format!("-I{}/lib/sdl/SDL2/include", nxdk_dir))
        .clang_arg(format!("-I{}/lib/winapi", nxdk_dir))
        .clang_arg(format!("-I{}/lib/xboxrt/vcruntime", nxdk_dir))
        .clang_arg(format!("-I{}/lib/net/lwip/src/include", nxdk_dir))
        .clang_arg(format!("-I{}/lib/net/nforceif/include", nxdk_dir))
        .clang_arg(format!("-I{}/lib/net/nvnetdrv", nxdk_dir))
        .clang_arg("-D__STDC__=1")
        .clang_arg("-DNXDK")
        .clang_arg("-DXBOX")
        .clang_arg("-DUSBH_USE_EXTERNAL_CONFIG=\"usbh_config_xbox.h\"")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .use_core()
        .ctypes_prefix("libc")
        .generate()
        .expect("Unable to generate bindings");

    println!("cargo:rerun-if-changed={}/lib/{}/{}/h", nxdk_dir, lib_path, header);

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable is not set");
    
    let mut out_path = std::path::PathBuf::from(&manifest_dir);
    out_path.push("src");
    out_path.push("bindings");

    bindings.write_to_file(out_path.join(format!("bindings_{}.rs", header))).expect("Unable to write bindings");
}


fn generate_mod_rs() -> std::io::Result<()> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable is not set");

    let bindings_dir = Path::new(&manifest_dir).join("src/bindings");
    let mut mod_rs_file = File::create(bindings_dir.join("mod.rs"))?;

    // Write the imports for each bindings file
    let bindings_files = fs::read_dir(bindings_dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "rs"))
        .filter(|entry| entry.file_name() != "mod.rs")
        .collect::<Vec<_>>();

    // Write the `mod.rs` content
    for entry in bindings_files {
        if let Some(file_name) = entry.path().file_stem() {
            let module_name = file_name.to_string_lossy();
            writeln!(mod_rs_file, "pub mod {};", module_name)?;
        }
    }

    Ok(())
}

fn main() {
    // TODO: Document this. This is here to prevent the IDE from building this file, as it will fail.
    let build_bindings = std::env::var("BUILD_BINDINGS").unwrap_or_else(|_| "false".to_string());

    if build_bindings != "true" {
        return;
    }

    let nxdk_dir;
    match std::env::var("NXDK_DIR") {
        Ok(v) => {
            nxdk_dir = v;
        },
        Err(e) => {
            panic!("Error reading NXDK_DIR environment variable: {:?}", e);
        }
    }

    println!("cargo:rustc-link-search=native={}/lib", nxdk_dir);
    println!("cargo:rustc-link-search=native={}/lib/xboxkrnl", nxdk_dir);

    link_lib("libpdclib");
    link_lib("libwinapi");
    link_lib("libnxdk");
    link_lib("libnxdk_hal");
    link_lib("libnxdk_net");
    link_lib("libnxdk_automount_d");
    link_lib("libzlib");
    link_lib("libxboxrt");
    link_lib("nxdk_usb");
    link_lib("libSDL2");
    link_lib("libSDL2_image");
    link_lib("libSDL_ttf");
    link_lib("libfreetype");
    link_lib("libjpeg");
    link_lib("libpng");
    link_lib("libpbkit");
    link_lib("libxboxkrnl");
    link_lib("winmm");

    gen_bindings(&nxdk_dir, "hal", "audio");
    gen_bindings(&nxdk_dir, "hal", "debug");
    gen_bindings(&nxdk_dir, "hal", "fileio");
    gen_bindings(&nxdk_dir, "hal", "led");
    gen_bindings(&nxdk_dir, "hal", "video");
    gen_bindings(&nxdk_dir, "hal", "xbox");
    
    gen_bindings(&nxdk_dir, "pbkit", "pbkit");

    gen_bindings(&nxdk_dir, "xboxkrnl", "xboxkrnl");

    gen_bindings(&nxdk_dir, "pdclib/include", "stdlib");
    gen_bindings(&nxdk_dir, "pdclib/include", "string");
    gen_bindings(&nxdk_dir, "pdclib/include", "stdio");
    gen_bindings(&nxdk_dir, "pdclib/include", "time");

    // USB
    gen_bindings(&nxdk_dir, "usb/libusbohci/inc", "usbh_lib");
    gen_bindings(&nxdk_dir, "usb/libusbohci/inc", "usbh_hid");
    gen_bindings(&nxdk_dir, "usb/libusbohci_xbox", "xid_driver");

    // SDL
    gen_bindings(&nxdk_dir, "sdl/SDL2/include", "SDL");

    gen_bindings(&nxdk_dir, "winapi", "windows");

    // Networking
    gen_bindings(&nxdk_dir, "net/lwip/src/include/lwip", "opt");
    gen_bindings(&nxdk_dir, "net/lwip/src/include/lwip", "api");
    gen_bindings(&nxdk_dir, "net/lwip/src/include/lwip", "netif");
    gen_bindings(&nxdk_dir, "net/lwip/src/include/lwip", "tcpip");
    gen_bindings(&nxdk_dir, "net/lwip/src/include/lwip", "tcp");

    // NXDK general helper functions
    gen_bindings(&nxdk_dir, "nxdk", "configsector");
    gen_bindings(&nxdk_dir, "nxdk", "fatx");
    gen_bindings(&nxdk_dir, "nxdk", "format");
    gen_bindings(&nxdk_dir, "nxdk", "mount");
    gen_bindings(&nxdk_dir, "nxdk", "net");
    gen_bindings(&nxdk_dir, "nxdk", "path");
    gen_bindings(&nxdk_dir, "nxdk", "xbe");

    if let Err(e) = generate_mod_rs() {
        eprintln!("Error generating mod.rs: {}", e);
    }
}
