use std::sync::Arc;
use ifascript::larql::{parse_query, LarqlEngine, OdùCorpus, OdùMetadata};
use ifascript::larql::schema::SensitivityLevel;
use ifascript::odu::ActionVessel;
use ifascript::odu::ODU_SET;

fn mock_corpus() -> Arc<OdùCorpus> {
    Arc::new(OdùCorpus {
        entries: vec![OdùMetadata {
            odu_id: 4,
            name: "Thunder Justice".into(),
            minimum_tier: 2,
            sensitivity_level: SensitivityLevel::High,
            version: "1.0".into(),
            confidence_baseline: 0.92,
            prescription_template: ActionVessel::Receipt,
            larql_tags: vec!["governance".into(), "accountability".into()],
            larql_rules: Default::default(),
            fractal_patterns: Default::default(),
            ripple_effect_score: 0.87,
            human_override_allowed: true,
        }],
    })
}

#[test]
fn test_parse_prepare() {
    assert!(parse_query(r#"PREPARE deploy CHECK: Consent"#).is_ok());
}

#[test]
fn test_parse_walk() {
    assert!(parse_query(r#"WALK last_7_days AGGREGATE: Receipt"#).is_ok());
}

#[test]
fn test_parse_synthesize() {
    assert!(parse_query(r#"SYNTHESIZE swarm FROM 4 WHERE confidence_baseline > 0.8"#).is_ok());
}

#[test]
fn test_parse_verify() {
    assert!(parse_query(r#"VERIFY Consent WHERE approved = TRUE"#).is_ok());
}

#[test]
fn test_parse_describe() {
    assert!(parse_query(r#"DESCRIBE growth AT SCALE micro,macro"#).is_ok());
}

#[test]
fn test_parse_invalid_query_fails() {
    assert!(parse_query("INVOKE something").is_err());
}

#[test]
fn test_execute_prepare_tier1_succeeds() {
    // Tier 1 is allowed to run PREPARE and VERIFY
    let corpus = mock_corpus();
    let engine = LarqlEngine::new(corpus, true, 1);
    let result = engine.execute(r#"PREPARE deploy CHECK: Consent"#).unwrap();
    assert!(result.confidence > 0.0);
    assert!(!result.mapped_vessels.is_empty());
}

#[test]
fn test_tier1_blocked_from_walk() {
    // Tier 1 cannot run WALK — must error
    let corpus = mock_corpus();
    let engine = LarqlEngine::new(corpus, true, 1);
    let result = engine.execute(r#"WALK last_7_days AGGREGATE: Receipt"#);
    assert!(result.is_err());
}

#[test]
fn test_execute_verify_passes() {
    let corpus = mock_corpus();
    let engine = LarqlEngine::new(corpus, false, 2);
    let result = engine.execute(r#"VERIFY Consent WHERE approved = TRUE"#).unwrap();
    assert!(result.action_steps[0].contains("verification passed"));
    assert!(result.mapped_vessels.contains(&ActionVessel::Consent));
}

#[test]
fn test_execute_verify_conditions_empty_passes() {
    let corpus = mock_corpus();
    let engine = LarqlEngine::new(corpus, false, 2);
    let result = engine.execute(r#"VERIFY Receipt"#).unwrap();
    assert!(result.action_steps[0].contains("verification passed"));
}

#[test]
fn test_synthesize_corpus_lookup() {
    let corpus = mock_corpus();
    let engine = LarqlEngine::new(corpus, false, 3);
    let result = engine.execute(
        r#"SYNTHESIZE governance FROM 4 WHERE confidence_baseline > 0.8"#
    ).unwrap();
    assert!(!result.action_steps.is_empty());
    assert!(result.mapped_vessels.contains(&ActionVessel::Receipt));
}

#[test]
fn test_safe_mode_blocks_low_confidence() {
    let mut corpus = mock_corpus();
    Arc::get_mut(&mut corpus).unwrap().entries[0].confidence_baseline = 0.5;
    let engine = LarqlEngine::new(corpus, true, 3);
    let result = engine.execute(r#"SYNTHESIZE test FROM 4"#);
    assert!(result.is_err());
}

#[test]
fn test_cache_hit_returns_same_result() {
    let corpus = mock_corpus();
    let engine = LarqlEngine::new(corpus, false, 2);
    let q = r#"VERIFY Consent WHERE approved = TRUE"#;
    let r1 = engine.execute(q).unwrap();
    let r2 = engine.execute(q).unwrap();
    assert_eq!(r1.action_steps, r2.action_steps);
    assert_eq!(r1.confidence, r2.confidence);
}

#[test]
fn test_walk_aggregates() {
    let corpus = mock_corpus();
    let engine = LarqlEngine::new(corpus, false, 2);
    let result = engine.execute(r#"WALK last_7_days AGGREGATE: Receipt"#).unwrap();
    assert!(!result.mapped_vessels.is_empty());
    assert!(result.confidence > 0.0);
}

#[test]
fn test_describe_returns_vision_vessel() {
    let corpus = mock_corpus();
    let engine = LarqlEngine::new(corpus, false, 2);
    let result = engine.execute(r#"DESCRIBE growth AT SCALE micro,macro"#).unwrap();
    assert!(result.mapped_vessels.contains(&ActionVessel::Vision));
    assert_eq!(result.action_steps.len(), 2);
}

// ── from_odu_set tests ────────────────────────────────────────────────────────

#[test]
fn test_from_odu_set_has_256_entries() {
    let corpus = OdùCorpus::from_odu_set();
    assert_eq!(corpus.entries.len(), ODU_SET.len());
}

#[test]
fn test_from_odu_set_entry_zero_is_genesis() {
    let corpus = OdùCorpus::from_odu_set();
    let meta = corpus.get(0).unwrap();
    assert_eq!(meta.prescription_template, ActionVessel::Genesis);
    assert_eq!(meta.minimum_tier, 1);
    assert!(meta.confidence_baseline > 0.9);
    assert!(meta.human_override_allowed);
}

#[test]
fn test_from_odu_set_synthesize_uses_default_corpus() {
    let corpus = Arc::new(OdùCorpus::from_odu_set());
    let engine = LarqlEngine::new(corpus, false, 3);
    let result = engine.execute(
        r#"SYNTHESIZE governance FROM 0 WHERE confidence_baseline > 0.8"#
    ).unwrap();
    assert!(!result.action_steps.is_empty());
    assert!(result.confidence > 0.8);
}

#[test]
fn test_from_odu_set_tier1_blocked_from_synthesize() {
    let corpus = Arc::new(OdùCorpus::from_odu_set());
    let engine = LarqlEngine::new(corpus, true, 1);
    let result = engine.execute(r#"SYNTHESIZE test FROM 0"#);
    assert!(result.is_err());
}

// ── condition_passes operator coverage ───────────────────────────────────────

#[test]
fn test_verify_gt_valid_threshold_passes() {
    let corpus = mock_corpus();
    let engine = LarqlEngine::new(corpus, false, 2);
    let result = engine.execute(r#"VERIFY Receipt WHERE confidence > 0.7"#).unwrap();
    assert!(result.action_steps[0].contains("passed"));
}

#[test]
fn test_verify_lte_passes() {
    let corpus = mock_corpus();
    let engine = LarqlEngine::new(corpus, false, 2);
    let result = engine.execute(r#"VERIFY Restraint WHERE score <= 1.0"#).unwrap();
    assert!(result.action_steps[0].contains("passed"));
}

#[test]
fn test_conditions_match_ripple_effect_score() {
    let corpus = Arc::new(OdùCorpus {
        entries: vec![OdùMetadata {
            odu_id: 10,
            name: "Test".into(),
            minimum_tier: 2,
            sensitivity_level: SensitivityLevel::Medium,
            version: "1.0".into(),
            confidence_baseline: 0.88,
            prescription_template: ActionVessel::Restraint,
            larql_tags: vec![],
            larql_rules: vec![],
            fractal_patterns: vec![],
            ripple_effect_score: 0.72,
            human_override_allowed: true,
        }],
    });
    let engine = LarqlEngine::new(corpus, false, 3);
    // ripple_effect_score 0.72 > 0.6 → should synthesize
    let result = engine.execute(
        r#"SYNTHESIZE test FROM 10 WHERE ripple_effect_score > 0.6"#
    ).unwrap();
    assert!(!result.action_steps.is_empty());
}

#[test]
fn test_conditions_match_human_override_allowed() {
    let corpus = Arc::new(OdùCorpus {
        entries: vec![OdùMetadata {
            odu_id: 11,
            name: "Test Override".into(),
            minimum_tier: 1,
            sensitivity_level: SensitivityLevel::Low,
            version: "1.0".into(),
            confidence_baseline: 0.90,
            prescription_template: ActionVessel::Consent,
            larql_tags: vec![],
            larql_rules: vec![],
            fractal_patterns: vec![],
            ripple_effect_score: 0.85,
            human_override_allowed: false,
        }],
    });
    let engine = LarqlEngine::new(corpus, false, 3);
    // human_override_allowed = false → condition `= TRUE` should exclude this ODU from synthesis
    let result = engine.execute(
        r#"SYNTHESIZE test FROM 11 WHERE human_override_allowed = TRUE"#
    ).unwrap();
    // No ODU matched — no "Synthesized wisdom" entry should appear
    assert!(!result.action_steps.iter().any(|s| s.contains("Synthesized wisdom")));
}
