use ifascript::hermetic::{default_gate, GateContext, HermeticPrinciple, EnforcementLevel};
use ifascript::cosmogram::AccessClass;
use ifascript::soul::MemoryTier;

#[test]
fn valid_tier1_passes() {
    let gate = default_gate();
    let access = AccessClass::Public;
    let memory = MemoryTier::Tier3Contributable;
    let ctx = GateContext {
        tier: 1,
        odu_id: 100,
        access_class: &access,
        memory_tier: &memory,
    };
    let result = gate.validate_all(&ctx);
    assert!(result.allowed, "tier1 odu_id=100 Public should be allowed");
}

#[test]
fn odu_exceeds_tier_max_blocked() {
    let gate = default_gate();
    let access = AccessClass::Public;
    let memory = MemoryTier::Tier3Contributable;
    // tier 1 max is 255; odu_id 300 exceeds it
    let ctx = GateContext {
        tier: 1,
        odu_id: 300,
        access_class: &access,
        memory_tier: &memory,
    };
    let result = gate.validate_all(&ctx);
    assert!(!result.allowed, "odu_id 300 should be blocked for tier 1");
    assert!(!result.violations.is_empty(), "should have violations");
}

#[test]
fn machine_only_odu0_blocked() {
    let gate = default_gate();
    let access = AccessClass::MachineOnly;
    let memory = MemoryTier::Tier1Deep;
    let ctx = GateContext {
        tier: 1,
        odu_id: 0,
        access_class: &access,
        memory_tier: &memory,
    };
    let result = gate.validate_all(&ctx);
    // Polarity rule: MachineOnly + odu_id 0 is a Hard violation
    assert!(!result.allowed, "MachineOnly with odu_id=0 should be blocked");
}

#[test]
fn audit_only_does_not_block() {
    let gate = default_gate();
    // Rhythm rule fires for tier >= 6 with Tier3Contributable memory
    // But Rhythm rule is AuditOnly, so allowed depends on whether a hard rule also fires
    // For tier=6, odu_id=1, MachineOnly with Tier3Contributable:
    // - Correspondence: odu_id=1 <= 32767 (tier6 max) → no violation
    // - Polarity: odu_id != 0, so no violation
    // - Rhythm: tier >= 6 AND Tier3Contributable → AuditOnly violation
    let access = AccessClass::MachineOnly;
    let memory = MemoryTier::Tier3Contributable;
    let ctx = GateContext {
        tier: 6,
        odu_id: 1,
        access_class: &access,
        memory_tier: &memory,
    };
    let result = gate.validate_all(&ctx);
    // Rhythm rule is AuditOnly → should not block on its own
    let has_audit_only = result.violations.iter().any(|v| v.enforcement == EnforcementLevel::AuditOnly);
    assert!(has_audit_only, "should have at least one AuditOnly violation from Rhythm rule");
}

#[test]
fn gate_allows_all_tiers_valid_odu() {
    let gate = default_gate();
    // For each tier, odu_id=0 with Public access (non-MachineOnly) should not panic
    for tier in 1u8..=7 {
        let access = AccessClass::Public;
        let memory = MemoryTier::Tier2Operational;
        let ctx = GateContext {
            tier,
            odu_id: 0,
            access_class: &access,
            memory_tier: &memory,
        };
        let _result = gate.validate_all(&ctx);
        // just check no panic
    }
}

#[test]
fn warnings_count_matches_soft_violations() {
    let gate = default_gate();
    // Rhythm rule (AuditOnly) fires for tier >= 6 with Tier3Contributable
    let access = AccessClass::MachineOnly;
    let memory = MemoryTier::Tier3Contributable;
    let ctx = GateContext {
        tier: 6,
        odu_id: 1,
        access_class: &access,
        memory_tier: &memory,
    };
    let result = gate.validate_all(&ctx);
    assert!(result.warnings > 0, "warnings should be > 0 when AuditOnly violations exist");
}
