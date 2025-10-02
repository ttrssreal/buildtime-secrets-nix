fn main() {
    let probe = system_deps::Config::new().probe().unwrap();

    cxx_build::bridge("src/lib.rs")
        .file("src/nix-wrap.cc")
        .includes(probe.all_include_paths())
        .std("c++23")
        .opt_level(1)
        .compile("cxxbridge-libnixstore");

    println!("cargo::rerun-if-changed=src/nix-wrap.cc");
    println!("cargo::rerun-if-changed=include/nix-wrap.hh");
}
