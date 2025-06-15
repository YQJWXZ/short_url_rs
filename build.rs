fn main() {
    // 编译 protobuf 文件到 src/pb 目录
    prost_build::Config::new()
        .out_dir("src/pb")
        .compile_protos(&["abi.proto"], &["."])
        .unwrap();

    // 告诉 cargo 当 proto 文件改变时重新构建
    println!("cargo:rerun-if-changed=abi.proto");
}
