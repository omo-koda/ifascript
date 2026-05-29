use ifascript::compiler::parser::{IfaParser, parse_full_program};

// ── v0.2 backward-compat: invocation-only programs ──────────────────────────

#[test]
fn divine_ifa_parses_without_error() {
    let src = include_str!("../examples/divine.ifa");
    let result = IfaParser::parse_program(src);
    assert!(result.is_ok(), "divine.ifa parse failed: {:?}", result.err());
    let invocations = result.unwrap();
    assert_eq!(invocations.len(), 5, "divine.ifa should have 5 invocations");
}

#[test]
fn full_program_invocations_match() {
    let src = include_str!("../examples/divine.ifa");
    let prog = parse_full_program(src).expect("parse_full_program should succeed");
    assert_eq!(prog.invocations.len(), 5);
    assert_eq!(prog.imports.len(), 0);
    assert_eq!(prog.definitions.len(), 0);
}

// ── import statements ────────────────────────────────────────────────────────

#[test]
fn import_statement_parses() {
    let src = r#"import "orisha/shango" as shango;"#;
    let prog = parse_full_program(src).expect("import parse");
    assert_eq!(prog.imports.len(), 1);
    assert_eq!(prog.imports[0].path, "orisha/shango");
    assert_eq!(prog.imports[0].alias.as_deref(), Some("shango"));
}

#[test]
fn multiple_imports_parse() {
    let src = r#"
        import "orisha/oshun" as oshun;
        import "orisha/ogun" as ogun;
    "#;
    let prog = parse_full_program(src).expect("multiple imports");
    assert_eq!(prog.imports.len(), 2);
}

// ── odù definitions ──────────────────────────────────────────────────────────

#[test]
fn odu_def_empty_body() {
    let src = r#"odù Ogbe<tier> {}"#;
    let prog = parse_full_program(src).expect("odu_def parse");
    assert_eq!(prog.definitions.len(), 1);
}

#[test]
fn odu_def_with_prescriptions() {
    let src = r#"
        odù Oyeku<level> {
            prescribe seal_boundaries();
            prescribe release_old(7);
        }
    "#;
    let prog = parse_full_program(src).expect("odu_def with prescriptions");
    assert_eq!(prog.definitions.len(), 1);
    if let ifascript::compiler::ast::Definition::Odu(def) = &prog.definitions[0] {
        assert_eq!(def.name, "Oyeku");
        assert_eq!(def.type_param, "level");
        assert_eq!(def.prescriptions.len(), 2);
        assert_eq!(def.prescriptions[0].action, "seal_boundaries");
        assert_eq!(def.prescriptions[1].action, "release_old");
    } else {
        panic!("expected OduDef");
    }
}

// ── ritual definitions ───────────────────────────────────────────────────────

#[test]
fn ritual_def_empty_body() {
    let src = r#"ritual daily_cast() {}"#;
    let prog = parse_full_program(src).expect("ritual empty body");
    assert_eq!(prog.definitions.len(), 1);
    if let ifascript::compiler::ast::Definition::Ritual(def) = &prog.definitions[0] {
        assert_eq!(def.name, "daily_cast");
        assert_eq!(def.params.len(), 0);
        assert_eq!(def.body.len(), 0);
    } else {
        panic!("expected RitualDef");
    }
}

#[test]
fn ritual_def_with_params_and_let() {
    let src = r#"
        ritual consecrate(amount: u64) {
            let total: u64 = 42;
        }
    "#;
    let prog = parse_full_program(src).expect("ritual with param and let");
    if let ifascript::compiler::ast::Definition::Ritual(def) = &prog.definitions[0] {
        assert_eq!(def.params.len(), 1);
        assert_eq!(def.params[0].name, "amount");
        assert_eq!(def.body.len(), 1);
    } else {
        panic!("expected RitualDef");
    }
}

// ── witness definitions ──────────────────────────────────────────────────────

#[test]
fn witness_def_parses() {
    let src = r#"witness council: 7 @"https://oracle.ifa" @"mainnet";"#;
    let prog = parse_full_program(src).expect("witness_def parse");
    assert_eq!(prog.definitions.len(), 1);
    if let ifascript::compiler::ast::Definition::Witness(def) = &prog.definitions[0] {
        assert_eq!(def.name, "council");
        assert_eq!(def.quorum, 7);
        assert_eq!(def.oracle, "https://oracle.ifa");
        assert_eq!(def.anchor, "mainnet");
    } else {
        panic!("expected WitnessDef");
    }
}

// ── full program: imports + definitions + invocations ───────────────────────

#[test]
fn full_program_all_sections() {
    let src = r#"
        import "stdlib/consent" as consent;

        ritual affirm(x: u8) {
            let flag: bool = true;
        }

        invoke daily_resonance;
        invoke thunder_justice with cause_effect:0.95 witness 3 settle Saturday;
    "#;
    let prog = parse_full_program(src).expect("full program parse");
    assert_eq!(prog.imports.len(), 1);
    assert_eq!(prog.definitions.len(), 1);
    assert_eq!(prog.invocations.len(), 2);
    assert_eq!(prog.invocations[1].witness_quorum, Some(3));
}

// ── statement types ──────────────────────────────────────────────────────────

#[test]
fn return_statement_parses() {
    let src = r#"
        ritual check() {
            return 1;
        }
    "#;
    let prog = parse_full_program(src).expect("return stmt");
    if let ifascript::compiler::ast::Definition::Ritual(def) = &prog.definitions[0] {
        assert_eq!(def.body.len(), 1);
        assert!(matches!(def.body[0], ifascript::compiler::ast::Statement::Return(_)));
    }
}

#[test]
fn if_statement_parses() {
    let src = r#"
        ritual gate_check() {
            if flag { return 1; } else { return 0; }
        }
    "#;
    let prog = parse_full_program(src).expect("if stmt");
    if let ifascript::compiler::ast::Definition::Ritual(def) = &prog.definitions[0] {
        assert_eq!(def.body.len(), 1);
        assert!(matches!(def.body[0], ifascript::compiler::ast::Statement::If(_)));
    }
}
