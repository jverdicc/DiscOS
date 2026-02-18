use discos_client::{canonical_output_matches_capsule, validate_claim_and_topic_ids, ClientError};

#[test]
fn validates_claim_and_topic_id_lengths() {
    let claim = [1u8; 32];
    let topic = [2u8; 32];
    assert!(validate_claim_and_topic_ids(&claim, &topic).is_ok());

    let err = validate_claim_and_topic_ids(&claim[..31], &topic).expect_err("claim length check");
    assert!(matches!(err, ClientError::InvalidInput(_)));

    let err = validate_claim_and_topic_ids(&claim, &topic[..31]).expect_err("topic length check");
    assert!(matches!(err, ClientError::InvalidInput(_)));
}

#[test]
fn canonical_output_must_equal_capsule_payload_fields() {
    let structured_output = br#"{\"a\":1}"#;
    let capsule = br#"{\"structured_output_hash_hex\":\"015abd7f5cc57a2dd94b7590f04ad8084273905ee33ec5cebeae62276a97f862\",\"claim_id_hex\":\"0101010101010101010101010101010101010101010101010101010101010101\",\"topic_id_hex\":\"0202020202020202020202020202020202020202020202020202020202020202\"}"#;
    assert!(
        canonical_output_matches_capsule(structured_output, capsule, &[1u8; 32], &[2u8; 32])
            .is_ok()
    );
}
