fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .type_attribute(
            "ServerAddressType",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile(&["../Protobufs/OpenBazaarApi.proto"], &["../Protobufs/"])?;
    Ok(())
}
