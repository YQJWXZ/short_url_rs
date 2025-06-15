fn main() {
    let mut config = prost_build::Config::new();
    config.bytes(["."]);
    config
        .out_dir("src/pb")
        .compile_protos(&["abi.proto"], &["."])
        .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=abi.proto");
}
