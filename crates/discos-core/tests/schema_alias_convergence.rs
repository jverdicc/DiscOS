use discos_core::topicid::{
    canonicalize_output_schema_id, CANONICAL_OUTPUT_SCHEMA_ID, OUTPUT_SCHEMA_ID_ALIASES,
};

#[test]
fn discos_schema_canonicalization_matches_evidenceos_accept_list() {
    assert_eq!(
        canonicalize_output_schema_id(CANONICAL_OUTPUT_SCHEMA_ID),
        CANONICAL_OUTPUT_SCHEMA_ID
    );

    for alias in OUTPUT_SCHEMA_ID_ALIASES {
        assert_eq!(
            canonicalize_output_schema_id(alias),
            CANONICAL_OUTPUT_SCHEMA_ID
        );
    }
}
