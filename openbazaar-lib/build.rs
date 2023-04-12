fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .type_attribute(
            "NodeAddressType",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile(&["../Protobufs/OpenBazaarRpc.proto"], &["../Protobufs/"])?;
    Ok(())
}
