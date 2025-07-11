use cmake::Config;

fn main() {
    let mut config = Config::new("luau");

    config
        .define("LUAU_BUILD_CLI", "OFF")
        .define("LUAU_BUILD_TESTS", "OFF")
        .define("LUAU_BUILD_WEB", "OFF")
        .define("LUAU_EXTERN_C", "ON")
        .define("LUAU_STATIC_CRT", "ON")
        .define("LUAU_ENABLE_ASSERT", "ON")
        .no_build_target(true)
        .profile(if cfg!(debug_assertions) {
            "RelWithDebInfo"
        } else {
            "Release"
        });

    #[cfg(target_os = "windows")]
    config.cxxflag("/EHsc");

    let dst = config.build();

    #[cfg(target_os = "windows")]
    {
        #[cfg(debug_assertions)]
        println!(
            "cargo:rustc-link-search=native={}/build/RelWithDebInfo",
            dst.display()
        );
        #[cfg(not(debug_assertions))]
        println!(
            "cargo:rustc-link-search=native={}/build/Release",
            dst.display()
        );
    }

    #[cfg(not(target_os = "windows"))]
    println!("cargo:rustc-link-search=native={}/build", dst.display());

    println!("cargo:rustc-link-lib=static=Luau.VM");
    println!("cargo:rustc-link-lib=static=Luau.Compiler");
    println!("cargo:rustc-link-lib=static=Luau.Ast");

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=stdc++");

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=c++");
}
