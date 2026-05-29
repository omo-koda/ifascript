use ifascript::ase_vault::{get_ese, get_prescriptions};

#[test]
fn principal_ese_myths_loaded() {
    for odu_id in 0u16..16 {
        let result = get_ese(1, odu_id);
        assert!(
            result.is_some(),
            "expected ese myth for odu_id {}, got None",
            odu_id
        );
        let myth = result.unwrap();
        assert!(!myth.is_empty(), "ese myth for odu_id {} is empty", odu_id);
    }
}

#[test]
fn eji_ogbe_ese_myth_content() {
    let myth = get_ese(1, 0).expect("Eji Ogbe should have an ese myth");
    assert!(
        myth.contains("Ẹjì Ogbe") || myth.contains("Eji Ogbe") || myth.contains("Light"),
        "Eji Ogbe myth should mention the first Odù or Light; got: {}",
        &myth[..myth.len().min(120)]
    );
}

#[test]
fn non_principal_returns_none() {
    assert!(
        get_ese(1, 16).is_none(),
        "odu_id 16 is not a principal meji — should return None"
    );
    assert!(get_ese(1, 255).is_none());
}

#[test]
fn tier_param_ignored_for_same_odu() {
    let t0 = get_ese(0, 0);
    let t1 = get_ese(1, 0);
    let t9 = get_ese(9, 0);
    assert_eq!(t0, t1, "tier param should not change result");
    assert_eq!(t1, t9, "tier param should not change result");
}

#[test]
fn prescriptions_loaded_for_principals() {
    for odu_id in 0u16..16 {
        let p = get_prescriptions(odu_id);
        assert!(
            p.is_some(),
            "expected prescriptions for odu_id {}",
            odu_id
        );
        let list = p.unwrap();
        assert!(!list.is_empty(), "prescriptions list for odu_id {} is empty", odu_id);
    }
}

#[test]
fn eji_ogbe_has_four_prescriptions() {
    let p = get_prescriptions(0).expect("Eji Ogbe prescriptions");
    assert_eq!(p.len(), 4, "expected 4 prescriptions for Eji Ogbe");
}

#[test]
fn prescriptions_non_principal_returns_none() {
    assert!(get_prescriptions(100).is_none());
}
