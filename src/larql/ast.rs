use crate::odu::ActionVessel;

#[derive(Debug, Clone)]
pub enum LarqlQuery {
    Prepare(PrepareQuery),
    Walk(WalkQuery),
    Synthesize(SynthesizeQuery),
    Verify(VerifyQuery),
    Describe(DescribeQuery),
}

#[derive(Debug, Clone)]
pub struct PrepareQuery {
    pub action: String,
    pub checks: Vec<CheckClause>,
    pub return_steps: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct WalkQuery {
    pub time_range: String,
    pub aggregates: Vec<AggregateClause>,
    pub compare_to: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SynthesizeQuery {
    pub context: String,
    pub sources: Vec<u16>,
    pub conditions: Vec<Condition>,
    pub confidence_threshold: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct VerifyQuery {
    pub vessel: ActionVessel,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone)]
pub struct DescribeQuery {
    pub target: String,
    pub scales: Vec<Scale>,
}

#[derive(Debug, Clone)]
pub struct CheckClause {
    pub vessel: ActionVessel,
    pub condition: Option<Condition>,
}

#[derive(Debug, Clone)]
pub struct AggregateClause {
    pub vessel: ActionVessel,
    pub condition: Option<Condition>,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub field: String,
    pub operator: Operator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    Eq,
    Gt,
    Lt,
    Gte,
    Lte,
    Contains,
    In,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scale {
    Micro,
    Meso,
    Macro,
}
