fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "windows" {
        // Only compile the C++ renderer on Windows
        cc::Build::new()
            .cpp(true)
            .file("src/render/Renderer.cpp")
            .compile("velowin_renderer");
            
        println!("cargo:rustc-link-lib=dcomp");
        println!("cargo:rustc-link-lib=d3d11");
        println!("cargo:rustc-link-lib=dxgi");
        println!("cargo:rustc-link-lib=d2d1");
    }
}
