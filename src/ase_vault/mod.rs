use std::collections::HashMap;
use serde::Deserialize;
use lazy_static::lazy_static;

#[derive(Deserialize)]
struct PrincipalJson {
    odu_id: u16,
    metadata: MetadataJson,
}

#[derive(Deserialize)]
struct MetadataJson {
    ese_myth: String,
    prescriptions: Vec<String>,
}

struct AseEntry {
    ese_myth: String,
    prescriptions: Vec<String>,
}

lazy_static! {
    static ref CORPUS: HashMap<u16, AseEntry> = {
        let raw: &[&str] = &[
            include_str!("../../data/16_principals/00_eji_ogbe.json"),
            include_str!("../../data/16_principals/01_oyeku_meji.json"),
            include_str!("../../data/16_principals/02_iwori_meji.json"),
            include_str!("../../data/16_principals/03_odi_meji.json"),
            include_str!("../../data/16_principals/04_irosun_meji.json"),
            include_str!("../../data/16_principals/05_owonrin_meji.json"),
            include_str!("../../data/16_principals/06_obara_meji.json"),
            include_str!("../../data/16_principals/07_okanran_meji.json"),
            include_str!("../../data/16_principals/08_ogunda_meji.json"),
            include_str!("../../data/16_principals/09_osa_meji.json"),
            include_str!("../../data/16_principals/10_ika_meji.json"),
            include_str!("../../data/16_principals/11_oturupon_meji.json"),
            include_str!("../../data/16_principals/12_otura_meji.json"),
            include_str!("../../data/16_principals/13_irete_meji.json"),
            include_str!("../../data/16_principals/14_ose_meji.json"),
            include_str!("../../data/16_principals/15_ofun_meji.json"),
        ];
        let mut map = HashMap::new();
        for s in raw {
            if let Ok(entry) = serde_json::from_str::<PrincipalJson>(s) {
                map.insert(entry.odu_id, AseEntry {
                    ese_myth: entry.metadata.ese_myth,
                    prescriptions: entry.metadata.prescriptions,
                });
            }
        }
        map
    };
}

/// Returns the ese (divination verse / myth) for the given odu_id.
/// `tier` is reserved for future access-gating; currently ignored.
/// Returns `Some` only for the 16 principal meji Odù (odu_id 0–15).
pub fn get_ese(tier: u8, odu_id: u16) -> Option<String> {
    let _ = tier;
    CORPUS.get(&odu_id).map(|e| e.ese_myth.clone())
}

/// Returns the actionable prescriptions for the given odu_id.
pub fn get_prescriptions(odu_id: u16) -> Option<Vec<String>> {
    CORPUS.get(&odu_id).map(|e| e.prescriptions.clone())
}
