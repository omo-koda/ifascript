use ifascript::cosmogram::{CosmogramEngine, Day, AccessClass, CastError};

#[test]
fn cast_basic() {
    let engine = CosmogramEngine::new();
    let result = engine.cast(1, 0, Day::Sunday, 1_000_000);
    let state = result.expect("cast should succeed");
    assert_eq!(state.odu_id, 0);
    assert_eq!(state.tier, 1);
}

#[test]
fn cast_invalid_tier() {
    let engine = CosmogramEngine::new();
    let err = engine.cast(0, 0, Day::Monday, 0).expect_err("tier 0 should be invalid");
    assert!(matches!(err, CastError::InvalidTier(0)));
}

#[test]
fn cast_odu_exceeds_tier_max() {
    // tier 1 max is 255; odu_id 256 should be blocked
    let engine = CosmogramEngine::new();
    let err = engine.cast(1, 256, Day::Monday, 0).expect_err("odu 256 should be blocked for tier 1");
    assert!(matches!(err, CastError::GateBlocked(_)));
}

#[test]
fn cast_all_tiers() {
    let engine = CosmogramEngine::new();
    for tier in 1u8..=7 {
        let result = engine.cast(tier, 0, Day::Friday, 0);
        assert!(result.is_ok(), "cast failed for tier {}", tier);
    }
}

#[test]
fn cast_validated_tier1_clean() {
    let engine = CosmogramEngine::new();
    // tier=1, odu_id=5, Wednesday, timestamp 1_700_000_000
    // odu_id 5 <= 255 (tier1 max), access_class=Public (not MachineOnly), odu_id != 0
    let result = engine.cast_validated(1, 5, Day::Wednesday, 1_700_000_000);
    assert!(result.is_ok(), "cast_validated should succeed: {:?}", result.err());
}

#[test]
fn entropy_hash_differs_per_input() {
    let engine = CosmogramEngine::new();
    let s1 = engine.cast(1, 10, Day::Tuesday, 0).expect("cast 1 ok");
    let s2 = engine.cast(1, 20, Day::Tuesday, 0).expect("cast 2 ok");
    assert_ne!(s1.entropy_hash, s2.entropy_hash, "different odu_ids should yield different hashes");
}

#[test]
fn window_open_field_is_bool() {
    let engine = CosmogramEngine::new();
    let state = engine.cast(1, 1, Day::Saturday, 0).expect("cast ok");
    // Just verify the field exists and has a bool type — no assertion on value
    let _: bool = state.window_open;
}

#[test]
fn from_entropy_constructor() {
    let engine = CosmogramEngine::from_entropy(vec![1, 2, 3]);
    let result = engine.cast(1, 0, Day::Sunday, 0);
    assert!(result.is_ok(), "engine from_entropy should produce a working engine");
}
