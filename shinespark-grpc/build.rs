fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let descriptor_path = std::path::PathBuf::from(&out_dir).join("file_descriptor_set.bin");

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(descriptor_path) // 이 경로에 메타데이터 파일을 생성
        .compile_protos(
            &["proto/hello.proto"], // .proto 파일 경로
            &["proto"],             // .proto 파일이 있는 디렉터리
        )?;

    Ok(())
}
