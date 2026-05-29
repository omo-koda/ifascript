use pest::Parser;
use pest_derive::Parser;
use crate::larql::ast::*;
use crate::larql::error::{LarqlError, LarqlResult};

#[derive(Parser)]
#[grammar = "src/larql/grammar.pest"]
pub struct LarqlParser;

pub fn parse_query(input: &str) -> LarqlResult<LarqlQuery> {
    let mut pairs = LarqlParser::parse(Rule::larql_query, input)
        .map_err(|e| LarqlError::Parse(e.to_string()))?;

    let pair = pairs.next().ok_or_else(|| LarqlError::Parse("Empty query".into()))?;

    match pair.as_rule() {
        Rule::prepare_query    => parse_prepare(pair),
        Rule::walk_query       => parse_walk(pair),
        Rule::synthesize_query => parse_synthesize(pair),
        Rule::verify_query     => parse_verify(pair),
        Rule::describe_query   => parse_describe(pair),
        _ => Err(LarqlError::Parse(format!("Unknown query type: {:?}", pair.as_rule()))),
    }
}

fn parse_prepare(pair: pest::iterators::Pair<Rule>) -> LarqlResult<LarqlQuery> {
    let mut action = None;
    let mut checks = Vec::new();
    let mut return_steps = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier if action.is_none() => {
                action = Some(inner.as_str().to_string());
            }
            Rule::checks => checks = parse_checks(inner)?,
            Rule::steps  => return_steps = Some(parse_steps(inner)?),
            _ => {}
        }
    }

    Ok(LarqlQuery::Prepare(PrepareQuery {
        action: action.ok_or_else(|| LarqlError::Parse("Missing action".into()))?,
        checks,
        return_steps,
    }))
}

fn parse_walk(pair: pest::iterators::Pair<Rule>) -> LarqlResult<LarqlQuery> {
    let mut time_range = None;
    let mut aggregates = Vec::new();
    let mut compare_to = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier if time_range.is_none() => {
                time_range = Some(inner.as_str().to_string());
            }
            Rule::aggregates => aggregates = parse_aggregates(inner)?,
            Rule::target     => compare_to = Some(inner.as_str().to_string()),
            _ => {}
        }
    }

    Ok(LarqlQuery::Walk(WalkQuery {
        time_range: time_range.ok_or_else(|| LarqlError::Parse("Missing time_range".into()))?,
        aggregates,
        compare_to,
    }))
}

fn parse_synthesize(pair: pest::iterators::Pair<Rule>) -> LarqlResult<LarqlQuery> {
    let mut context = None;
    let mut sources = Vec::new();
    let mut conditions = Vec::new();
    let mut confidence_threshold = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier if context.is_none() => {
                context = Some(inner.as_str().to_string());
            }
            Rule::odu_id => {
                let id = inner.as_str().parse::<u16>()
                    .map_err(|_| LarqlError::Parse(format!("Invalid odu_id: {}", inner.as_str())))?;
                sources.push(id);
            }
            Rule::conditions => conditions = parse_conditions(inner)?,
            Rule::number if confidence_threshold.is_none() => {
                confidence_threshold = inner.as_str().parse::<f64>().ok();
            }
            _ => {}
        }
    }

    Ok(LarqlQuery::Synthesize(SynthesizeQuery {
        context: context.ok_or_else(|| LarqlError::Parse("Missing context".into()))?,
        sources,
        conditions,
        confidence_threshold,
    }))
}

fn parse_verify(pair: pest::iterators::Pair<Rule>) -> LarqlResult<LarqlQuery> {
    let mut vessel = None;
    let mut conditions = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier if vessel.is_none() => {
                vessel = Some(inner.as_str().parse::<crate::odu::ActionVessel>()
                    .map_err(LarqlError::UnknownVessel)?);
            }
            Rule::conditions => conditions = parse_conditions(inner)?,
            _ => {}
        }
    }

    Ok(LarqlQuery::Verify(VerifyQuery {
        vessel: vessel.ok_or_else(|| LarqlError::Parse("Missing vessel".into()))?,
        conditions,
    }))
}

fn parse_describe(pair: pest::iterators::Pair<Rule>) -> LarqlResult<LarqlQuery> {
    let mut target = None;
    let mut scales = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier if target.is_none() => {
                target = Some(inner.as_str().to_string());
            }
            Rule::scale => {
                let scale = match inner.as_str() {
                    "micro" => Scale::Micro,
                    "meso"  => Scale::Meso,
                    "macro" => Scale::Macro,
                    _       => Scale::Micro,
                };
                scales.push(scale);
            }
            _ => {}
        }
    }

    Ok(LarqlQuery::Describe(DescribeQuery {
        target: target.ok_or_else(|| LarqlError::Parse("Missing target".into()))?,
        scales,
    }))
}

fn parse_checks(pair: pest::iterators::Pair<Rule>) -> LarqlResult<Vec<CheckClause>> {
    pair.into_inner().map(parse_check).collect()
}

fn parse_check(pair: pest::iterators::Pair<Rule>) -> LarqlResult<CheckClause> {
    let mut vessel = None;
    let mut condition = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier if vessel.is_none() => {
                vessel = Some(inner.as_str().parse::<crate::odu::ActionVessel>()
                    .map_err(LarqlError::UnknownVessel)?);
            }
            Rule::condition => condition = Some(parse_condition(inner)?),
            _ => {}
        }
    }

    Ok(CheckClause {
        vessel: vessel.ok_or_else(|| LarqlError::Parse("Missing vessel in check".into()))?,
        condition,
    })
}

fn parse_aggregates(pair: pest::iterators::Pair<Rule>) -> LarqlResult<Vec<AggregateClause>> {
    pair.into_inner().map(parse_aggregate).collect()
}

fn parse_aggregate(pair: pest::iterators::Pair<Rule>) -> LarqlResult<AggregateClause> {
    let mut vessel = None;
    let mut condition = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier if vessel.is_none() => {
                vessel = Some(inner.as_str().parse::<crate::odu::ActionVessel>()
                    .map_err(LarqlError::UnknownVessel)?);
            }
            Rule::condition => condition = Some(parse_condition(inner)?),
            _ => {}
        }
    }

    Ok(AggregateClause {
        vessel: vessel.ok_or_else(|| LarqlError::Parse("Missing vessel in aggregate".into()))?,
        condition,
    })
}

fn parse_conditions(pair: pest::iterators::Pair<Rule>) -> LarqlResult<Vec<Condition>> {
    pair.into_inner().map(parse_condition).collect()
}

fn parse_condition(pair: pest::iterators::Pair<Rule>) -> LarqlResult<Condition> {
    let mut field    = None;
    let mut operator = None;
    let mut value    = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier if field.is_none() => {
                field = Some(inner.as_str().to_string());
            }
            Rule::operator => operator = Some(parse_operator(inner.as_str())?),
            Rule::string | Rule::number | Rule::boolean => {
                value = Some(parse_value(inner)?);
            }
            _ => {}
        }
    }

    Ok(Condition {
        field:    field.ok_or_else(|| LarqlError::Parse("Missing field".into()))?,
        operator: operator.ok_or_else(|| LarqlError::Parse("Missing operator".into()))?,
        value:    value.ok_or_else(|| LarqlError::Parse("Missing value".into()))?,
    })
}

fn parse_operator(s: &str) -> LarqlResult<Operator> {
    match s {
        "="        => Ok(Operator::Eq),
        ">"        => Ok(Operator::Gt),
        "<"        => Ok(Operator::Lt),
        ">="       => Ok(Operator::Gte),
        "<="       => Ok(Operator::Lte),
        "CONTAINS" => Ok(Operator::Contains),
        "IN"       => Ok(Operator::In),
        _          => Err(LarqlError::UnknownOperator(s.to_string())),
    }
}

fn parse_value(pair: pest::iterators::Pair<Rule>) -> LarqlResult<serde_json::Value> {
    match pair.as_rule() {
        Rule::string  => Ok(serde_json::Value::String(
            pair.as_str().trim_matches('"').to_string()
        )),
        Rule::number  => pair.as_str().parse::<f64>()
            .map_err(|_| LarqlError::Parse("Invalid number".into()))
            .and_then(|n| serde_json::Number::from_f64(n)
                .ok_or_else(|| LarqlError::Parse("Non-finite number".into()))
                .map(serde_json::Value::Number)),
        Rule::boolean => Ok(serde_json::Value::Bool(
            pair.as_str().eq_ignore_ascii_case("TRUE")
        )),
        _ => Err(LarqlError::Parse(format!("Unknown value type: {:?}", pair.as_rule()))),
    }
}

fn parse_steps(pair: pest::iterators::Pair<Rule>) -> LarqlResult<Vec<String>> {
    let content = pair.as_str();
    serde_json::from_str(content)
        .map_err(|e| LarqlError::Parse(format!("Invalid steps JSON: {}", e)))
}
