use super::*;

#[test]
fn methods_deserialize_from_the_renderer_contract() {
    let parse =
        |value: &str| serde_json::from_value::<ServiceMethod>(Value::String(value.to_owned()));

    assert_eq!(parse("GET").expect("GET"), ServiceMethod::Get);
    assert_eq!(parse("POST").expect("POST"), ServiceMethod::Post);
    assert_eq!(parse("PATCH").expect("PATCH"), ServiceMethod::Patch);
    assert_eq!(parse("DELETE").expect("DELETE"), ServiceMethod::Delete);
    assert!(parse("PUT").is_err());
    assert!(parse("get").is_err());
}

#[test]
fn packets_preserve_status_and_encode_chunks_as_base64() {
    let started = serde_json::to_value(ServicePacket::Started { status: 429 }).expect("status");
    let chunk =
        serde_json::to_value(ServicePacket::chunk(&[0, 159, 146, 150, 255])).expect("chunk");

    assert_eq!(
        started,
        serde_json::json!({ "type": "started", "status": 429 })
    );
    assert_eq!(
        chunk,
        serde_json::json!({ "type": "chunk", "bytes": "AJ+Slv8=" })
    );
}
