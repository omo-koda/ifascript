use crate::odu::ActionVessel;

pub type ExecutableStep = String;

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub action_steps: Vec<ExecutableStep>,
    pub mapped_vessels: Vec<ActionVessel>,
    pub confidence: f64,
    pub human_override_required: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
