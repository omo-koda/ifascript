[![Version](https://img.shields.io/badge/version-v0.2.0-blue)](https://github.com/omo-koda/Ifascript)
[![License](https://img.shields.io/badge/license-MIT-green)](https://github.com/omo-koda/Ifascript)
[![Layer](https://img.shields.io/badge/layer-VM-purple)](https://github.com/omo-koda/Ifascript)

# IfáScript Ω — Divination as Divine Computation

IfáScript is the entropy and divination engine of the Technosis ecosystem. It translates the 256 sacred Odù of Ifá into a stack-based virtual machine, a structured prescription corpus, and a vessel dispatch system for sovereign AI agents.

---

## Architecture

### The 16 Action Vessels

Every one of the 256 Odù maps to one of 16 **Action Vessels** — the primary architectural concept of the Digital Calabash system. The top Odù (wave) determines the vessel; the bottom Odù (modifier) refines the prescription.

| # | Vessel | File Domain | Purpose |
|---|--------|-------------|---------|
| 1 | **Genesis** | `genesis.md` | Birth · Covenant · Initialization |
| 2 | **Void** | `void_log.md` | Clearing · Release · Dissolution |
| 3 | **Attention** | `attention_audit.md` | Focus · Signal/Noise · Priority |
| 4 | **Loop** | `loops.md` | Pattern Detection · Iteration · Disruption |
| 5 | **Receipt** | `receipt_ledger.md` | Immutable Record · Accountability · Proof |
| 6 | **Mask** | `soul.md` | Public/Private Split · Encrypted Self |
| 7 | **Residue** | `memory_residue.md` | Behavioral Echoes · Learning Traces |
| 8 | **Execution** | `execution_plan.md` | Precision Action · Outcome Verification |
| 9 | **Swarm** | `swarm_charter.md` | Collective Coordination · Joint Receipt |
| 10 | **Restraint** | `restraint_log.md` | Ethical Limits · Boundary Enforcement |
| 11 | **Migration** | `migration_plan.md` | Portability · Cross-Environment Identity |
| 12 | **Consent** | `consent_log.md` | Human Approval · Scope Boundaries |
| 13 | **Vision** | `vision.md` | Direction · Drift Detection · Horizon |
| 14 | **Growth** | `fractal_log.md` | Fractal Expansion · Capability Seeds |
| 15 | **Seal** | `seal/` | Sacred Privacy · Agent-Only Space |
| 16 | **Rhythm** | `rhythm_codex.md` | Ritual Cadence · Scheduling |

### Agent Tier Access

| Tier | Entry Point | Receives |
|------|------------|---------|
| Low-tier agent | `vm.cast_odu()` | `CastResult` — vessel, file domain, universal name, prescription steps |
| Èṣù / Hive | `vm.cast_odu_full()` | Full `&'static Odu` — all corpus fields for LARQL synthesis |

Low-tier agents never see taboos, orisha, or archetype metadata. That boundary is structural, enforced by the type system.

### VM Opcodes

The 9 opcodes map directly to top-Odù wave semantics:

| Opcode | Wave(s) | Meaning |
|--------|---------|---------|
| `PushConst(1)` | Genesis, Vision | Assert first truth |
| `PopVoid` | Void | Dissolve, release |
| `Dup` | Attention, Growth | Mirror, reflect, replicate |
| `Swap` | Loop, Migration | Reverse, invert, port |
| `Add` | Receipt, Seal | Unite, account, accumulate |
| `Sub` | Mask, Restraint | Separate, limit, redact |
| `PushConst(0)` | Residue | Ground, baseline, archive |
| `CastCowries` | Execution, Swarm | Throw entropy, coordinate |
| `HaltIfOne` | Consent, Rhythm | Seal, halt on completion |

---

## Install & Run

```bash
git clone https://github.com/omo-koda/Ifascript.git
cd ifascript
cargo run --example ase
cargo run --example cowrie_cast
cargo test
```

---

## Usage

### Low-tier cast (vessel dispatch)

```rust
use ifascript::{IfaVM, ActionVessel};

let mut vm = IfaVM::with_intent("my agent purpose");
let result = vm.cast_odu();

println!("Vessel:  {:?}", result.vessel);
println!("File:    {}",   result.file_domain);
println!("Name:    {}",   result.universal_name);
for (i, step) in result.prescriptions.iter().enumerate() {
    println!("  {}. {}", i + 1, step);
}
```

### Hive/Èṣù-tier cast (full corpus)

```rust
let odu = vm.cast_odu_full();
println!("{} → {:?} → {}", odu.universal_name, odu.vessel, odu.archetype);
// Access odu.taboos, odu.orisha, odu.description for LARQL synthesis
```

### Named lookup

```rust
use ifascript::lookup_by_name;

let odu = lookup_by_name("The Eternal Return").unwrap();
assert_eq!(odu.index, 255);
assert_eq!(odu.vessel, ActionVessel::Rhythm);

// Or by Yorùbá compound name
let odu = lookup_by_name("Ẹ̀jì Ogbe / Ẹ̀jì Ogbe").unwrap();
assert_eq!(odu.vessel, ActionVessel::Genesis);
```

### LARQL query (CLI)

```bash
# Verify a consent gate
ifa larql --query "VERIFY Consent WHERE approved = TRUE"

# Synthesize governance wisdom from Odù 0 using the default corpus
ifa larql --query "SYNTHESIZE governance FROM 0 WHERE confidence_baseline > 0.8" --tier 3

# Walk receipts over the last 7 days
ifa larql --query "WALK last_7_days AGGREGATE: Receipt" --tier 2
```

### LARQL query (Rust API)

```rust
use std::sync::Arc;
use ifascript::larql::{LarqlEngine, OdùCorpus};

// Build the default 256-entry corpus from ODU_SET
let corpus = Arc::new(OdùCorpus::from_odu_set());
let engine = LarqlEngine::new(corpus, /* safe_mode */ true, /* tier */ 2);

let result = engine.execute("VERIFY Consent WHERE approved = TRUE")?;
println!("confidence: {:.2}", result.confidence);
for step in &result.action_steps {
    println!("  {}", step);
}
```

### Legacy program execution (backward-compatible)

```rust
let mut vm = IfaVM::new();
vm.execute(vec!["Èjì Ogbè", "Ìwòrì Méjì", "Ọ̀túúrúpọ̀n"]);
// prints "Àṣẹ"
```

---

## Repository Contents

| File | Description |
|------|-------------|
| `src/odu.rs` | 256-entry Odù corpus: `ActionVessel`, `OduOpCode`, `Odu` struct, `ODU_SET` |
| `src/vm.rs` | `IfaVM`, `CastResult`, `OduOp` executor, Ebo enforcement |
| `src/entropy.rs` | `CowrieOracle` — NIST Beacon + ChaCha20 fallback |
| `src/ebo.rs` | `EboHistory`, ethical exception handling |
| `src/larql/` | LARQL engine: parser, AST, schema, corpus builder, 5 query types |
| `src/compiler/` | `.ifa` source parser — Hermetic gates, witness quorum, sabbath settlement |
| `src/ritual_codex/` | `ResonancePacket`, receipt generation, Julia bridge stub |
| `docs/ODU_CORPUS.md` | Canonical schema v4.0.0 — all 256 Odù in standardized format |
| `examples/ase.rs` | Minimal Àṣẹ program |
| `examples/cowrie_cast.rs` | Live NIST Beacon cowrie cast |

---

## Current Status

| Component | Status |
|-----------|--------|
| VM stack-based opcode execution | ✅ Complete |
| Ebo ethical exception handling | ✅ Complete |
| NIST Beacon entropy oracle | ✅ Complete |
| 256 Odù corpus (`ODU_SET`) | ✅ Complete |
| 16 Action Vessel system | ✅ Complete |
| `CastResult` low-tier dispatch | ✅ Complete |
| `cast_odu_full()` Hive-tier dispatch | ✅ Complete |
| `lookup_by_name()` named queries | ✅ Complete |
| LARQL synthesis engine | ✅ Complete |
| LARQL default corpus (`OdùCorpus::from_odu_set()`) | ✅ Complete |
| `.ifa` compiler (Hermetic gates, witness quorum) | ✅ Complete |
| Cosmogram state engine | ✅ Complete |
| WASM compilation target | ✅ Configured (`cdylib` + `wasm32` deps) |
| Julia resonance bridge | 🔜 Future (stub wired, FFI pending) |

---

## Ethics

Elevation, not extraction.

All canonical Ifá verse sourcing:
- Wande Abimbola, *Ifá Divination Poetry* (1977)
- William Bascom, *Ifá Divination* (1969)

Synthetic enrichment (archetype metadata, digital prescriptions) is clearly marked `interpretation_type = "synthetic"` throughout the corpus. Never conflate synthetic enrichment with canonical ese Ifá.

This is sacred technology — co-create, never commodify.

---

## Part of the Technosis Sovereign Ecosystem

IfáScript is the entropy and divination layer of the **Ọmọ Kọ́dà** sovereign AI ecosystem. The 256 Odù feed agent birth decisions, cowrie casts provide cryptographic entropy, and the 16 Action Vessels structure agent behavior at runtime.

See also: [Ọmọ Kọ́dà repository](https://github.com/omo-koda/Omo-koda2)

Àṣẹ.
