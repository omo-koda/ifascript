// ifascript/src/compiler/parser.rs
// Ògún's Forge: Parser — invocations (v0.2 compat) + full program (v0.3)

use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;
use crate::compiler::ast;

#[derive(Parser)]
#[grammar = "src/compiler/grammar.pest"]
pub struct IfaParser;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedInvocation {
    pub ritual_name: String,
    pub gate_principle: Option<String>,
    pub gate_threshold: Option<f64>,
    pub witness_quorum: Option<u8>,
    pub sabbath: Option<String>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Parse failed: {0}")]
    Pest(#[from] pest::error::Error<Rule>),
    #[error("Missing ritual name")]
    MissingRitualName,
}

impl IfaParser {
    /// Parse a full .ifa program string — returns all invocations
    pub fn parse_program(input: &str) -> Result<Vec<ParsedInvocation>, ParseError> {
        let program = Self::parse(Rule::program, input)?
            .next()
            .expect("program rule always present");

        program.into_inner()
            .filter(|p| p.as_rule() == Rule::invocation)
            .map(parse_invocation)
            .collect()
    }
}

fn parse_invocation(
    pair: pest::iterators::Pair<Rule>,
) -> Result<ParsedInvocation, ParseError> {
    let mut ritual_name = None;
    let mut gate_principle = None;
    let mut gate_threshold = None;
    let mut witness_quorum = None;
    let mut sabbath = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => ritual_name = Some(inner.as_str().to_string()),

            // hermetic_principle is a silent rule — it folds into gate_spec's span.
            // Split gate_spec's raw text on ':' to recover both parts.
            Rule::gate_spec => {
                let raw = inner.as_str();
                if let Some(colon) = raw.rfind(':') {
                    gate_principle = Some(raw[..colon].trim().to_string());
                    gate_threshold = raw[colon + 1..].trim().parse().ok();
                }
            }

            // witness_spec = { "witness" ~ number } — number is the only child
            Rule::witness_spec => {
                witness_quorum = inner
                    .into_inner()
                    .next()
                    .and_then(|p| p.as_str().parse().ok());
            }

            Rule::sabbath_spec => {
                let raw = inner.as_str();
                sabbath = Some(if raw.starts_with('"') && raw.ends_with('"') {
                    raw[1..raw.len() - 1].replace("\\\"", "\"")
                } else {
                    raw.to_string()
                });
            }

            _ => {}
        }
    }

    Ok(ParsedInvocation {
        ritual_name: ritual_name.ok_or(ParseError::MissingRitualName)?,
        gate_principle,
        gate_threshold,
        witness_quorum,
        sabbath,
    })
}

// ── Full program parser (v0.3) ────────────────────────────────────────────────

/// Parse a full .ifa program into a typed `Program` AST.
pub fn parse_full_program(input: &str) -> Result<ast::Program, ParseError> {
    let tree = IfaParser::parse(Rule::program, input)?
        .next()
        .expect("program rule always present");

    let mut imports     = Vec::new();
    let mut definitions = Vec::new();
    let mut invocations = Vec::new();

    for pair in tree.into_inner() {
        match pair.as_rule() {
            Rule::import_stmt  => imports.push(build_import(pair)),
            Rule::definition   => definitions.push(build_definition(pair)?),
            Rule::invocation   => invocations.push(build_ast_invocation(pair)?),
            Rule::EOI          => {}
            _                  => {}
        }
    }

    Ok(ast::Program { imports, definitions, invocations })
}

fn build_import(pair: pest::iterators::Pair<Rule>) -> ast::ImportStmt {
    let mut path  = String::new();
    let mut alias = None;
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::string => path  = unquote(inner.as_str()),
            Rule::ident  => alias = Some(inner.as_str().to_string()),
            _ => {}
        }
    }
    ast::ImportStmt { path, alias }
}

fn build_definition(pair: pest::iterators::Pair<Rule>) -> Result<ast::Definition, ParseError> {
    let inner = pair.into_inner().next().expect("definition has one child");
    match inner.as_rule() {
        Rule::odu_def     => Ok(ast::Definition::Odu(build_odu_def(inner)?)),
        Rule::ritual_def  => Ok(ast::Definition::Ritual(build_ritual_def(inner)?)),
        Rule::witness_def => Ok(ast::Definition::Witness(build_witness_def(inner))),
        _ => unreachable!("unexpected definition variant"),
    }
}

fn build_odu_def(pair: pest::iterators::Pair<Rule>) -> Result<ast::OduDef, ParseError> {
    let mut name       = String::new();
    let mut type_param = String::new();
    let mut prescriptions = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::odu_name          => name       = inner.as_str().to_string(),
            Rule::ident             => type_param = inner.as_str().to_string(),
            Rule::prescription_stmt => prescriptions.push(build_prescription(inner)),
            _ => {}
        }
    }
    Ok(ast::OduDef { name, type_param, prescriptions })
}

fn build_ritual_def(pair: pest::iterators::Pair<Rule>) -> Result<ast::RitualDef, ParseError> {
    let mut name       = String::new();
    let mut params     = Vec::new();
    let mut attributes = Vec::new();
    let mut body       = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident      => name = inner.as_str().to_string(),
            Rule::param_list => params = build_param_list(inner),
            Rule::attr_list  => attributes = build_attr_list(inner),
            Rule::block      => body = build_block(inner),
            _ => {}
        }
    }
    Ok(ast::RitualDef { name, params, attributes, body })
}

fn build_witness_def(pair: pest::iterators::Pair<Rule>) -> ast::WitnessDef {
    let mut name   = String::new();
    let mut quorum = 0u8;
    let mut oracle = String::new();
    let mut anchor = String::new();
    let mut strings_seen = 0usize;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident  => name   = inner.as_str().to_string(),
            Rule::number => quorum = inner.as_str().parse().unwrap_or(0),
            Rule::string => {
                let val = unquote(inner.as_str());
                if strings_seen == 0 { oracle = val; } else { anchor = val; }
                strings_seen += 1;
            }
            _ => {}
        }
    }
    ast::WitnessDef { name, quorum, oracle, anchor }
}

fn build_param_list(pair: pest::iterators::Pair<Rule>) -> Vec<ast::Param> {
    pair.into_inner()
        .filter(|p| p.as_rule() == Rule::param)
        .map(|p| {
            let mut name = String::new();
            let mut typ  = ast::TypeExpr::Primitive(ast::PrimitiveType::U8);
            for inner in p.into_inner() {
                match inner.as_rule() {
                    Rule::ident     => name = inner.as_str().to_string(),
                    Rule::type_expr => typ  = build_type_expr(inner),
                    _ => {}
                }
            }
            ast::Param { name, typ }
        })
        .collect()
}

fn build_attr_list(pair: pest::iterators::Pair<Rule>) -> Vec<ast::Attribute> {
    pair.into_inner()
        .filter(|p| p.as_rule() == Rule::attribute)
        .map(|p| {
            let mut name  = String::new();
            let mut value = None;
            for inner in p.into_inner() {
                match inner.as_rule() {
                    Rule::ident   => name  = inner.as_str().to_string(),
                    Rule::literal => value = Some(build_literal(inner)),
                    _ => {}
                }
            }
            ast::Attribute { name, value }
        })
        .collect()
}

fn build_type_expr(pair: pest::iterators::Pair<Rule>) -> ast::TypeExpr {
    let inner = pair.into_inner().next().expect("type_expr has child");
    match inner.as_rule() {
        Rule::primitive_type => ast::TypeExpr::Primitive(match inner.as_str() {
            "u8"     => ast::PrimitiveType::U8,
            "u16"    => ast::PrimitiveType::U16,
            "u32"    => ast::PrimitiveType::U32,
            "u64"    => ast::PrimitiveType::U64,
            "bool"   => ast::PrimitiveType::Bool,
            "string" => ast::PrimitiveType::StringT,
            _        => ast::PrimitiveType::U8,
        }),
        Rule::odu_type => {
            let mut parts = inner.into_inner();
            let name  = parts.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let param = parts.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            ast::TypeExpr::OduType { name, param }
        }
        Rule::generic_type => {
            let mut parts = inner.into_inner();
            let name  = parts.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let param = parts.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            ast::TypeExpr::Generic { name, param }
        }
        _ => ast::TypeExpr::Primitive(ast::PrimitiveType::U8),
    }
}

fn build_block(pair: pest::iterators::Pair<Rule>) -> Vec<ast::Statement> {
    pair.into_inner()
        .filter(|p| p.as_rule() == Rule::statement)
        .map(build_statement)
        .collect()
}

fn build_statement(pair: pest::iterators::Pair<Rule>) -> ast::Statement {
    let inner = pair.into_inner().next().expect("statement has child");
    match inner.as_rule() {
        Rule::prescription_stmt => ast::Statement::Prescription(build_prescription(inner)),
        Rule::let_stmt          => build_let_stmt(inner),
        Rule::if_stmt           => build_if_stmt(inner),
        Rule::return_stmt       => {
            let expr = inner.into_inner().next().map(build_expression);
            ast::Statement::Return(expr)
        }
        _ => ast::Statement::Return(None),
    }
}

fn build_prescription(pair: pest::iterators::Pair<Rule>) -> ast::PrescriptionStmt {
    let mut action = String::new();
    let mut args   = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident    => action = inner.as_str().to_string(),
            Rule::arg_list => args = inner.into_inner()
                .filter(|p| p.as_rule() == Rule::expression)
                .filter_map(|p| {
                    let a = p.into_inner().next()?;
                    if a.as_rule() == Rule::atom {
                        let v = a.into_inner().next()?;
                        if v.as_rule() == Rule::literal {
                            return Some(build_literal(v));
                        }
                    }
                    None
                })
                .collect(),
            _ => {}
        }
    }
    ast::PrescriptionStmt { action, args }
}

fn build_let_stmt(pair: pest::iterators::Pair<Rule>) -> ast::Statement {
    let mut name  = String::new();
    let mut typ   = ast::TypeExpr::Primitive(ast::PrimitiveType::U8);
    let mut value = ast::Expression::Literal(ast::Literal::Number(0.0));

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident      => name  = inner.as_str().to_string(),
            Rule::type_expr  => typ   = build_type_expr(inner),
            Rule::expression => value = build_expression(inner),
            _ => {}
        }
    }
    ast::Statement::Let(ast::LetStmt { name, typ, value })
}

fn build_if_stmt(pair: pest::iterators::Pair<Rule>) -> ast::Statement {
    let mut condition  = ast::Expression::Literal(ast::Literal::Bool(true));
    let mut then_block = Vec::new();
    let mut else_block = None;
    let mut blocks_seen = 0usize;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expression => condition = build_expression(inner),
            Rule::block => {
                let stmts = build_block(inner);
                if blocks_seen == 0 { then_block = stmts; }
                else                { else_block = Some(stmts); }
                blocks_seen += 1;
            }
            _ => {}
        }
    }
    ast::Statement::If(ast::IfStmt { condition, then_block, else_block })
}

fn build_expression(pair: pest::iterators::Pair<Rule>) -> ast::Expression {
    let mut children: Vec<_> = pair.into_inner().collect();

    if children.len() == 1 {
        return build_atom(children.remove(0));
    }

    // left-assoc: fold  atom op atom op atom …
    let mut iter = children.into_iter();
    let first = build_atom(iter.next().expect("first atom"));
    let mut acc = first;
    while let (Some(op_pair), Some(rhs_pair)) = (iter.next(), iter.next()) {
        let op  = build_binary_op(op_pair.as_str());
        let rhs = build_atom(rhs_pair);
        acc = ast::Expression::BinaryOp {
            left:  Box::new(acc),
            op,
            right: Box::new(rhs),
        };
    }
    acc
}

fn build_atom(pair: pest::iterators::Pair<Rule>) -> ast::Expression {
    let inner = pair.into_inner().next().expect("atom has child");
    match inner.as_rule() {
        Rule::call_expr => {
            let mut parts = inner.into_inner();
            let name = parts.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let args = parts.map(build_expression).collect();
            ast::Expression::Call { name, args }
        }
        Rule::odu_lit => {
            let mut parts = inner.into_inner();
            let name  = parts.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let param = parts.next().map(|p| Box::new(build_expression(p)));
            ast::Expression::OduLiteral {
                name,
                param: param.map(|e| {
                    if let ast::Expression::Literal(l) = *e {
                        Box::new(l)
                    } else {
                        Box::new(ast::Literal::Number(0.0))
                    }
                }),
            }
        }
        Rule::literal    => ast::Expression::Literal(build_literal(inner)),
        Rule::expression => build_expression(inner),
        Rule::ident      => ast::Expression::Ident(inner.as_str().to_string()),
        _                => ast::Expression::Literal(ast::Literal::Bool(false)),
    }
}

fn build_literal(pair: pest::iterators::Pair<Rule>) -> ast::Literal {
    let inner = pair.into_inner().next().expect("literal has child");
    match inner.as_rule() {
        Rule::string   => ast::Literal::Str(unquote(inner.as_str())),
        Rule::bool_lit => ast::Literal::Bool(inner.as_str() == "true"),
        Rule::number   => ast::Literal::Number(inner.as_str().parse().unwrap_or(0.0)),
        Rule::odu_name => ast::Literal::OduName(inner.as_str().to_string()),
        _              => ast::Literal::Bool(false),
    }
}

fn build_binary_op(s: &str) -> ast::BinaryOp {
    match s {
        "+"  => ast::BinaryOp::Add,
        "-"  => ast::BinaryOp::Sub,
        "*"  => ast::BinaryOp::Mul,
        "/"  => ast::BinaryOp::Div,
        "==" => ast::BinaryOp::Eq,
        "!=" => ast::BinaryOp::Neq,
        "<"  => ast::BinaryOp::Lt,
        ">"  => ast::BinaryOp::Gt,
        _    => ast::BinaryOp::Add,
    }
}

fn build_ast_invocation(pair: pest::iterators::Pair<Rule>) -> Result<ast::Invocation, ParseError> {
    let parsed = parse_invocation(pair)?;
    Ok(ast::Invocation {
        ritual_name: parsed.ritual_name,
        gate: parsed.gate_principle.zip(parsed.gate_threshold).map(|(p, t)| ast::GateSpec {
            principle: ast::HermeticPrinciple::from_str(&p)
                .unwrap_or(ast::HermeticPrinciple::Mentalism),
            threshold: t,
        }),
        witness_quorum: parsed.witness_quorum,
        sabbath: parsed.sabbath.map(|s| ast::SabbathSpec::from_str(&s)),
    })
}

fn unquote(s: &str) -> String {
    if s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].replace("\\\"", "\"")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Vec<ParsedInvocation> {
        IfaParser::parse_program(s).expect("parse should succeed")
    }

    #[test]
    fn simple_invoke() {
        let r = parse("invoke thunder_justice;");
        assert_eq!(r[0].ritual_name, "thunder_justice");
        assert_eq!(r[0].gate_principle, None);
        assert_eq!(r[0].witness_quorum, None);
    }

    #[test]
    fn with_gate() {
        let r = parse("invoke t with cause_effect:0.95;");
        assert_eq!(r[0].gate_principle, Some("cause_effect".into()));
        assert_eq!(r[0].gate_threshold, Some(0.95));
    }

    #[test]
    fn with_witness_only() {
        // Witness without gate must parse correctly
        let r = parse("invoke t witness 3;");
        assert_eq!(r[0].witness_quorum, Some(3));
        assert_eq!(r[0].gate_principle, None);
    }

    #[test]
    fn full_invoke() {
        let r = parse("invoke t with cause_effect:0.95 witness 3 settle Saturday;");
        let i = &r[0];
        assert_eq!(i.ritual_name, "t");
        assert_eq!(i.gate_principle, Some("cause_effect".into()));
        assert_eq!(i.gate_threshold, Some(0.95));
        assert_eq!(i.witness_quorum, Some(3));
        assert_eq!(i.sabbath, Some("Saturday".into()));
    }

    #[test]
    fn keyword_as_ident_fails() {
        // Reserved keywords cannot be used as ritual names
        assert!(IfaParser::parse_program("invoke invoke;").is_err());
        assert!(IfaParser::parse_program("invoke witness;").is_err());
        assert!(IfaParser::parse_program("invoke settle;").is_err());
        assert!(IfaParser::parse_program("invoke ritual;").is_err());
    }

    #[test]
    fn multiple_invocations() {
        let r = parse("invoke alpha; invoke beta witness 2;");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].ritual_name, "alpha");
        assert_eq!(r[1].ritual_name, "beta");
        assert_eq!(r[1].witness_quorum, Some(2));
    }

    #[test]
    fn empty_program() {
        assert_eq!(parse("").len(), 0);
    }

    #[test]
    fn line_comment_skipped() {
        let r = parse("// Ṣàngó's justice\ninvoke thunder_justice;");
        assert_eq!(r[0].ritual_name, "thunder_justice");
    }

    #[test]
    fn block_comment_skipped() {
        let r = parse("/* opening */ invoke test;");
        assert_eq!(r[0].ritual_name, "test");
    }

    #[test]
    fn settle_any() {
        let r = parse("invoke r settle any;");
        assert_eq!(r[0].sabbath, Some("any".into()));
    }

    #[test]
    fn settle_quoted_string() {
        let r = parse(r#"invoke r settle "custom day";"#);
        assert_eq!(r[0].sabbath, Some("custom day".into()));
    }

    #[test]
    fn all_principles() {
        for p in &["mentalism", "correspondence", "vibration",
                   "polarity", "rhythm", "cause_effect", "gender"] {
            let src = format!("invoke r with {}:0.5;", p);
            let result = parse(&src);
            assert_eq!(
                result[0].gate_principle.as_deref(),
                Some(*p),
                "failed for {}",
                p
            );
        }
    }

    #[test]
    fn missing_semicolon_fails() {
        assert!(IfaParser::parse_program("invoke ritual").is_err());
    }
}
