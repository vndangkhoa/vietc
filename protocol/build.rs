fn main() {
    #[cfg(feature = "x11")]
    {
        println!("cargo:rustc-link-lib=X11");
        println!("cargo:rustc-link-lib=Xtst");

        if let Ok(_) = pkg_config::probe_library("x11") {}
        if let Ok(_) = pkg_config::probe_library("xtst") {}
    }
}
