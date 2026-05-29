use ifascript::ritual_codex::{RitualCodex, ResonancePacket};
use ifascript::cosmogram::{CosmogramEngine, Day};
use ifascript::hermetic::HermeticPrinciple;

#[test]
fn cast_resonance_tier1_succeeds() {
    let engine = CosmogramEngine::new();
    let codex = RitualCodex::new();
    let packet = ResonancePacket::new(5, 1, Day::Wednesday, 1_700_000_000, "test intent");
    let result = codex.cast_resonance(packet, &engine);
    assert!(result.is_ok(), "cast_resonance should succeed for tier=1, odu_id=5: {:?}", result.err());
}

#[test]
fn receipt_has_entropy_hash() {
    let engine = CosmogramEngine::new();
    let codex = RitualCodex::new();
    let packet = ResonancePacket::new(5, 1, Day::Wednesday, 1_700_000_000, "entropy test");
    let receipt = codex.cast_resonance(packet, &engine).expect("cast should succeed");
    assert!(!receipt.entropy_hash.is_empty(), "entropy_hash should be non-empty");
}

#[test]
fn receipt_gates_passed() {
    let engine = CosmogramEngine::new();
    let codex = RitualCodex::new();
    let packet = ResonancePacket::new(5, 1, Day::Wednesday, 1_700_000_000, "gates test");
    let receipt = codex.cast_resonance(packet, &engine).expect("cast should succeed");
    assert!(receipt.gates_passed, "gates_passed should be true for valid cast");
}

#[test]
fn receipt_has_dominant_orisha() {
    let engine = CosmogramEngine::new();
    let codex = RitualCodex::new();
    let packet = ResonancePacket::new(5, 1, Day::Wednesday, 1_700_000_000, "orisha test");
    let receipt = codex.cast_resonance(packet, &engine).expect("cast should succeed");
    assert!(receipt.orisha_dominant.is_some(), "orisha_dominant should be Some");
}

#[test]
fn cast_resonance_invalid_tier_fails() {
    let engine = CosmogramEngine::new();
    let codex = RitualCodex::new();
    let packet = ResonancePacket::new(0, 0, Day::Sunday, 0, "invalid tier");
    let result = codex.cast_resonance(packet, &engine);
    assert!(result.is_err(), "cast_resonance with tier=0 should return Err");
}

#[test]
fn ritual_codex_default() {
    let codex = RitualCodex::default();
    assert!(codex.data_dir.is_none(), "data_dir should be None by default");
}

#[test]
fn packet_gate_bias() {
    let mut packet = ResonancePacket::new(5, 1, Day::Monday, 0, "bias test");
    packet.set_gate_bias(HermeticPrinciple::Mentalism, 0.9);
    let bias = packet.gate_bias();
    assert!(bias.contains_key(&HermeticPrinciple::Mentalism), "gate_bias should contain Mentalism key");
}
