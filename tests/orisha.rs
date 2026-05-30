use ifascript::orisha::{OrishaVector, Orisha};
use ifascript::cosmogram::Day;

#[test]
fn default_is_uniform() {
    let v = OrishaVector::default();
    let vals = [v.esu, v.ogun, v.oya, v.obatala, v.oshun, v.shango, v.yemoja];
    let first = vals[0];
    for &val in &vals {
        assert!((val - first).abs() < 0.01, "default values should be approximately equal");
    }
}

#[test]
fn from_archetype_esu_dominant() {
    let v = OrishaVector::from_archetype("esu");
    assert_eq!(v.dominant(), Some(Orisha::Esu));
}

#[test]
fn from_archetype_shango_dominant() {
    let v = OrishaVector::from_archetype("shango");
    assert_eq!(v.dominant(), Some(Orisha::Shango));
}

#[test]
fn from_odu_day_deterministic() {
    let v1 = OrishaVector::from_odu_day(0, &Day::Sunday);
    let v2 = OrishaVector::from_odu_day(0, &Day::Sunday);
    assert_eq!(v1.esu, v2.esu);
    assert_eq!(v1.ogun, v2.ogun);
    assert_eq!(v1.oya, v2.oya);
    assert_eq!(v1.obatala, v2.obatala);
    assert_eq!(v1.oshun, v2.oshun);
    assert_eq!(v1.shango, v2.shango);
    assert_eq!(v1.yemoja, v2.yemoja);
}

#[test]
fn normalize_sums_to_one() {
    let mut v = OrishaVector::from_archetype("ogun");
    v.normalize();
    let sum = v.esu + v.ogun + v.oya + v.obatala + v.oshun + v.shango + v.yemoja;
    assert!((sum - 1.0).abs() < 0.0001, "normalized sum should be ~1.0, got {}", sum);
}

#[test]
fn scale_halves_all_fields() {
    let mut v = OrishaVector::from_archetype("esu");
    let original_esu = v.esu;
    v.scale(0.5);
    assert!((v.esu - original_esu * 0.5).abs() < 1e-10, "esu should be halved");
}

#[test]
fn dominant_returns_highest() {
    let v = OrishaVector {
        esu: 0.1,
        ogun: 0.1,
        oya: 0.1,
        obatala: 0.9,
        oshun: 0.1,
        shango: 0.1,
        yemoja: 0.1,
    };
    assert_eq!(v.dominant(), Some(Orisha::Obatala));
}
