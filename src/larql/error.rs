use thiserror::Error;

#[derive(Error, Debug)]
pub enum LarqlError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Unknown ActionVessel: {0}")]
    UnknownVessel(String),

    #[error("Unknown operator: {0}")]
    UnknownOperator(String),

    #[error("Missing required field: {0} in vessel {1}")]
    MissingField(String, String),

    #[error("Tier {0} not authorized for query type {1}")]
    TierUnauthorized(u8, String),

    #[error("Safe mode blocked unreviewed pattern: {0}")]
    SafeModeBlocked(String),

    #[error("Corpus lookup failed for Odu ID {0}")]
    CorpusLookupFailed(u16),

    #[error("Condition evaluation error: {0}")]
    ConditionEval(String),

    #[error("Cache error: {0}")]
    Cache(String),
}

pub type LarqlResult<T> = Result<T, LarqlError>;
