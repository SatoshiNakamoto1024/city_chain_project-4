fn main() -> Result<(), Box<dyn std::error::Error>> {
    // immudb.proto ファイルをコンパイルしてRustコードを生成
    tonic_build::configure()
        .build_server(true)  // サーバーコードを生成
        .build_client(true)  // クライアントコードを生成
        .compile(&["proto/immudb.proto"], &["proto"])?;  // プロトコルバッファのパスを指定

    Ok(())
}
