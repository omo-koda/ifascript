pub use crate::odu::ActionVessel;

#[derive(Debug, Clone, PartialEq)]
pub enum SensitivityLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct OdùMetadata {
    pub odu_id: u16,
    pub name: String,
    pub minimum_tier: u8,
    pub sensitivity_level: SensitivityLevel,
    pub version: String,
    pub confidence_baseline: f64,
    pub prescription_template: ActionVessel,
    pub larql_tags: Vec<String>,
    pub larql_rules: Vec<String>,
    pub fractal_patterns: Vec<String>,
    pub ripple_effect_score: f64,
    pub human_override_allowed: bool,
}

#[derive(Debug)]
pub struct OdùCorpus {
    pub entries: Vec<OdùMetadata>,
}

impl OdùCorpus {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Build a default corpus from the static ODU_SET, deriving metadata from each Odù's
    /// tier, vessel, hermetic gate, and prescription data.
    pub fn from_odu_set() -> Self {
        use crate::odu::ODU_SET;

        let entries = ODU_SET.iter().map(|odu| {
            let sensitivity_level = match odu.tier {
                1 => SensitivityLevel::Low,
                2 => SensitivityLevel::Medium,
                3 => SensitivityLevel::High,
                _ => SensitivityLevel::Critical,
            };
            let confidence_baseline = match odu.tier {
                1 => 0.92,
                2 => 0.85,
                _ => 0.78,
            };
            let larql_tags = vec![
                odu.archetype.to_lowercase(),
                odu.domain.to_lowercase(),
                odu.hermetic_gate.to_string(),
            ];
            let larql_rules = odu.prescriptions.iter()
                .map(|p| p.to_string())
                .collect();

            OdùMetadata {
                odu_id: odu.index as u16,
                name: odu.universal_name.to_string(),
                minimum_tier: odu.tier,
                sensitivity_level,
                version: "1.0".to_string(),
                confidence_baseline,
                prescription_template: odu.vessel,
                larql_tags,
                larql_rules,
                fractal_patterns: vec![odu.hermetic_gate.to_string()],
                ripple_effect_score: confidence_baseline - 0.05,
                human_override_allowed: odu.tier <= 2,
            }
        }).collect();

        Self { entries }
    }

    pub fn get(&self, odu_id: u16) -> Option<&OdùMetadata> {
        self.entries.iter().find(|e| e.odu_id == odu_id)
    }
}

impl Default for OdùCorpus {
    fn default() -> Self {
        Self::new()
    }
}
