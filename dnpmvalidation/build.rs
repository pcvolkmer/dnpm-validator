fn main() {
    cxx_build::bridge("src/lib.rs") // returns a cc::Build
        .std("c++23")
        .compile("dnpmvalidation");

    println!("cargo:rerun-if-changed=src/lib.rs");
}
