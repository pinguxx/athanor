use athanor_core::envelope::Envelope;
use serde_json::json;

#[test]
fn test_envelope_creation_and_payload() {
    let envelope =
        Envelope::new(json!({ "transmutation": "gold" })).with_correlation_id("corr-12345");

    assert_eq!(envelope.correlation_id, Some("corr-12345".into()));
    assert!(envelope.timestamp_ms > 0);
    assert_eq!(envelope.payload()["transmutation"], "gold");

    let mut envelope = envelope;
    envelope.payload_mut()["purity"] = json!("99.99%");
    assert_eq!(envelope.payload()["purity"], "99.99%");

    let mapped = envelope.map(|p| p["transmutation"].as_str().unwrap_or("").to_uppercase());
    assert_eq!(mapped.payload, "GOLD");
    assert_eq!(mapped.correlation_id, Some("corr-12345".into()));

    let inner = mapped.into_inner();
    assert_eq!(inner, "GOLD");
}

#[test]
fn test_envelope_serde_roundtrip() {
    let original = Envelope::new("alchemical message".to_string()).with_correlation_id("test-id");

    let serialized = serde_json::to_string(&original).expect("serialize envelope");
    let deserialized: Envelope<String> =
        serde_json::from_str(&serialized).expect("deserialize envelope");

    assert_eq!(original.id, deserialized.id);
    assert_eq!(original.timestamp_ms, deserialized.timestamp_ms);
    assert_eq!(original.correlation_id, deserialized.correlation_id);
    assert_eq!(original.payload, deserialized.payload);
}
