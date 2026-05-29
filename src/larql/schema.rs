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

    pub fn get(&self, odu_id: u16) -> Option<&OdùMetadata> {
        self.entries.iter().find(|e| e.odu_id == odu_id)
    }
}

impl Default for OdùCorpus {
    fn default() -> Self {
        Self::new()
    }
}
