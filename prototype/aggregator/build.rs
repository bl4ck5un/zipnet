fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../common/src/dc_proto.proto")?;
    Ok(())
}
