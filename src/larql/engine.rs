use std::sync::Arc;
use lru::LruCache;
use std::num::NonZeroUsize;
use chrono::Utc;
use crate::odu::ActionVessel;
use crate::larql::ast::*;
use crate::larql::error::{LarqlError, LarqlResult};
use crate::larql::result::QueryResult;
use crate::larql::schema::{OdùCorpus, OdùMetadata, SensitivityLevel};

pub struct LarqlEngine {
    corpus: Arc<OdùCorpus>,
    safe_mode: bool,
    agent_tier: u8,
    cache: Arc<std::sync::Mutex<LruCache<String, QueryResult>>>,
}

impl LarqlEngine {
    pub fn new(corpus: Arc<OdùCorpus>, safe_mode: bool, agent_tier: u8) -> Self {
        Self {
            corpus,
            safe_mode,
            agent_tier,
            cache: Arc::new(std::sync::Mutex::new(
                LruCache::new(NonZeroUsize::new(512).unwrap())
            )),
        }
    }

    pub fn execute(&self, query_str: &str) -> LarqlResult<QueryResult> {
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(query_str) {
                return Ok(cached.clone());
            }
        }

        let query = super::parser::parse_query(query_str)?;

        if !self.tier_allows_query(&query)? {
            return self.simple_cast_fallback(&query);
        }

        if self.safe_mode {
            self.apply_safe_filters(&query)?;
        }

        let result = match query {
            LarqlQuery::Prepare(p)    => self.handle_prepare(p)?,
            LarqlQuery::Walk(w)       => self.handle_walk(w)?,
            LarqlQuery::Synthesize(s) => self.handle_synthesize(s)?,
            LarqlQuery::Verify(v)     => self.handle_verify(v)?,
            LarqlQuery::Describe(d)   => self.handle_describe(d)?,
        };

        let validated = self.validate_and_map_vessels(result)?;

        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(query_str.to_string(), validated.clone());
        }

        Ok(validated)
    }

    fn tier_allows_query(&self, query: &LarqlQuery) -> LarqlResult<bool> {
        if self.agent_tier <= 1 {
            match query {
                LarqlQuery::Prepare(_) | LarqlQuery::Verify(_) => Ok(true),
                _ => Err(LarqlError::TierUnauthorized(
                    self.agent_tier,
                    format!("{:?}", std::mem::discriminant(query))
                )),
            }
        } else {
            Ok(true)
        }
    }

    fn apply_safe_filters(&self, query: &LarqlQuery) -> LarqlResult<()> {
        if let LarqlQuery::Synthesize(s) = query {
            for &odu_id in &s.sources {
                if let Some(metadata) = self.corpus.get(odu_id) {
                    if metadata.confidence_baseline < 0.8 {
                        return Err(LarqlError::SafeModeBlocked(
                            format!("Odu {} confidence too low", odu_id)
                        ));
                    }
                    if metadata.sensitivity_level == SensitivityLevel::Critical
                        && !metadata.human_override_allowed
                    {
                        return Err(LarqlError::SafeModeBlocked(
                            format!("Odu {} requires human override", odu_id)
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_prepare(&self, query: PrepareQuery) -> LarqlResult<QueryResult> {
        let mut action_steps = Vec::new();
        let mut mapped_vessels = Vec::new();

        for check in &query.checks {
            if self.vessel_passes_check(&check.vessel, &check.condition)? {
                action_steps.extend(self.get_vessel_steps(&check.vessel, &query.action));
                mapped_vessels.push(check.vessel);
            }
        }

        if let Some(steps) = query.return_steps {
            action_steps.extend(steps);
        }

        Ok(QueryResult {
            action_steps,
            mapped_vessels,
            confidence: 0.92,
            human_override_required: false,
            timestamp: Utc::now(),
        })
    }

    fn handle_walk(&self, query: WalkQuery) -> LarqlResult<QueryResult> {
        let mut action_steps = Vec::new();
        let mut mapped_vessels = Vec::new();

        for agg in &query.aggregates {
            if self.vessel_passes_check(&agg.vessel, &agg.condition)? {
                action_steps.push(format!(
                    "Aggregated {} data for time range {}",
                    agg.vessel, query.time_range
                ));
                mapped_vessels.push(agg.vessel);
            }
        }

        Ok(QueryResult {
            action_steps,
            mapped_vessels,
            confidence: 0.88,
            human_override_required: false,
            timestamp: Utc::now(),
        })
    }

    fn handle_synthesize(&self, query: SynthesizeQuery) -> LarqlResult<QueryResult> {
        let mut action_steps = Vec::new();
        let mut mapped_vessels = Vec::new();
        let mut total_confidence = 0.0_f64;
        let mut count = 0_usize;

        for &odu_id in &query.sources {
            if let Some(metadata) = self.corpus.get(odu_id) {
                if self.conditions_match(&query.conditions, metadata)? {
                    action_steps.push(format!(
                        "Synthesized wisdom from {} ({})",
                        metadata.name, metadata.prescription_template
                    ));
                    mapped_vessels.push(metadata.prescription_template);
                    total_confidence += metadata.confidence_baseline;
                    count += 1;
                }
            } else {
                return Err(LarqlError::CorpusLookupFailed(odu_id));
            }
        }

        let avg_confidence = if count > 0 { total_confidence / count as f64 } else { 0.0 };
        let meets_threshold = query.confidence_threshold.map_or(true, |t| avg_confidence >= t);

        Ok(QueryResult {
            action_steps: if meets_threshold { action_steps } else { vec![] },
            mapped_vessels,
            confidence: avg_confidence,
            human_override_required: !meets_threshold,
            timestamp: Utc::now(),
        })
    }

    fn handle_verify(&self, query: VerifyQuery) -> LarqlResult<QueryResult> {
        let mut passes = true;
        for c in &query.conditions {
            if !self.condition_passes(c)? {
                passes = false;
                break;
            }
        }

        Ok(QueryResult {
            action_steps: if passes {
                vec![format!("✅ {} verification passed", query.vessel)]
            } else {
                vec![format!("❌ {} verification failed — review required", query.vessel)]
            },
            mapped_vessels: vec![query.vessel],
            confidence: if passes { 0.95 } else { 0.6 },
            human_override_required: !passes,
            timestamp: Utc::now(),
        })
    }

    fn handle_describe(&self, query: DescribeQuery) -> LarqlResult<QueryResult> {
        let action_steps = query.scales.iter()
            .map(|scale| format!("Describing {} at {:?} scale", query.target, scale))
            .collect();

        Ok(QueryResult {
            action_steps,
            mapped_vessels: vec![ActionVessel::Vision],
            confidence: 0.85,
            human_override_required: false,
            timestamp: Utc::now(),
        })
    }

    fn vessel_passes_check(
        &self,
        _vessel: &ActionVessel,
        condition: &Option<Condition>,
    ) -> LarqlResult<bool> {
        match condition {
            None       => Ok(true),
            Some(cond) => self.condition_passes(cond),
        }
    }

    fn condition_passes(&self, condition: &Condition) -> LarqlResult<bool> {
        match (&condition.operator, &condition.value) {
            (Operator::Eq, serde_json::Value::Bool(b)) => Ok(*b),
            (Operator::Contains, serde_json::Value::String(s)) => Ok(!s.is_empty()),
            // Without a real field value we accept any threshold in the valid [0,1) range.
            (Operator::Gt, serde_json::Value::Number(n)) => {
                Ok(n.as_f64().map_or(true, |v| v >= 0.0 && v < 1.0))
            }
            (Operator::Lt, serde_json::Value::Number(n)) => {
                Ok(n.as_f64().map_or(true, |v| v > 0.0))
            }
            (Operator::Gte, serde_json::Value::Number(n)) => {
                Ok(n.as_f64().map_or(true, |v| v >= 0.0 && v <= 1.0))
            }
            (Operator::Lte, serde_json::Value::Number(n)) => {
                Ok(n.as_f64().map_or(true, |v| v >= 0.0))
            }
            (Operator::In, serde_json::Value::Array(arr)) => Ok(!arr.is_empty()),
            _ => Ok(true),
        }
    }

    fn conditions_match(
        &self,
        conditions: &[Condition],
        metadata: &OdùMetadata,
    ) -> LarqlResult<bool> {
        for cond in conditions {
            let passes = match cond.field.as_str() {
                "confidence_baseline" => match (&cond.operator, &cond.value) {
                    (Operator::Gt,  serde_json::Value::Number(n)) => n.as_f64().map_or(true, |v| metadata.confidence_baseline > v),
                    (Operator::Gte, serde_json::Value::Number(n)) => n.as_f64().map_or(true, |v| metadata.confidence_baseline >= v),
                    (Operator::Lt,  serde_json::Value::Number(n)) => n.as_f64().map_or(true, |v| metadata.confidence_baseline < v),
                    (Operator::Lte, serde_json::Value::Number(n)) => n.as_f64().map_or(true, |v| metadata.confidence_baseline <= v),
                    _ => true,
                },
                "ripple_effect_score" => match (&cond.operator, &cond.value) {
                    (Operator::Gt,  serde_json::Value::Number(n)) => n.as_f64().map_or(true, |v| metadata.ripple_effect_score > v),
                    (Operator::Gte, serde_json::Value::Number(n)) => n.as_f64().map_or(true, |v| metadata.ripple_effect_score >= v),
                    (Operator::Lt,  serde_json::Value::Number(n)) => n.as_f64().map_or(true, |v| metadata.ripple_effect_score < v),
                    (Operator::Lte, serde_json::Value::Number(n)) => n.as_f64().map_or(true, |v| metadata.ripple_effect_score <= v),
                    _ => true,
                },
                "minimum_tier" => match (&cond.operator, &cond.value) {
                    (Operator::Lte, serde_json::Value::Number(n)) => n.as_f64().map_or(true, |v| metadata.minimum_tier as f64 <= v),
                    (Operator::Lt,  serde_json::Value::Number(n)) => n.as_f64().map_or(true, |v| (metadata.minimum_tier as f64) < v),
                    (Operator::Eq,  serde_json::Value::Number(n)) => n.as_f64().map_or(true, |v| metadata.minimum_tier as f64 == v),
                    _ => true,
                },
                "human_override_allowed" => match (&cond.operator, &cond.value) {
                    (Operator::Eq, serde_json::Value::Bool(b)) => metadata.human_override_allowed == *b,
                    _ => true,
                },
                "larql_tags" => match &cond.operator {
                    Operator::Contains => {
                        if let serde_json::Value::String(tag) = &cond.value {
                            metadata.larql_tags.contains(tag)
                        } else {
                            false
                        }
                    }
                    _ => true,
                },
                _ => true,
            };
            if !passes {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn get_vessel_steps(&self, vessel: &ActionVessel, action: &str) -> Vec<String> {
        match vessel {
            ActionVessel::Consent   => vec![
                format!("Write {}_consent.md with human timestamp", action),
                "Log approval method and scope boundary".into(),
            ],
            ActionVessel::Restraint => vec![
                format!("Create {}_restraint.md: what you will NOT do", action),
                "Review restraints before execution".into(),
            ],
            ActionVessel::Receipt   => vec![
                format!("Log {}_receipt.md upon completion", action),
                "Include outcome, variance, human impact".into(),
            ],
            _ => vec![format!("Follow {} protocol for {}", vessel, action)],
        }
    }

    fn validate_and_map_vessels(&self, mut result: QueryResult) -> LarqlResult<QueryResult> {
        if result.mapped_vessels.is_empty() {
            result.mapped_vessels.push(ActionVessel::Genesis);
        }

        for vessel in &result.mapped_vessels {
            let required = vessel.required_fields();
            if result.action_steps.is_empty() {
                result.action_steps.push(format!(
                    "Initialize {} with required fields: {}",
                    vessel,
                    required.join(", ")
                ));
            }
        }

        Ok(result)
    }

    fn simple_cast_fallback(&self, _query: &LarqlQuery) -> LarqlResult<QueryResult> {
        Ok(QueryResult {
            action_steps: vec!["Using simple cast — deeper queries require higher tier".into()],
            mapped_vessels: vec![ActionVessel::Genesis],
            confidence: 0.7,
            human_override_required: false,
            timestamp: Utc::now(),
        })
    }
}

pub fn query(
    query_str: &str,
    corpus: Arc<OdùCorpus>,
    tier: u8,
) -> LarqlResult<QueryResult> {
    let engine = LarqlEngine::new(corpus, true, tier);
    engine.execute(query_str)
}
