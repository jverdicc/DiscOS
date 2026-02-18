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
fn canonical_output_must_equal_capsule() {
    assert!(canonical_output_matches_capsule(&[1, 2, 3], &[1, 2, 3]).is_ok());
    let err = canonical_output_matches_capsule(&[1, 2, 3], &[1, 2, 4]).expect_err("mismatch");
    assert!(matches!(err, ClientError::VerificationFailed(_)));
}
