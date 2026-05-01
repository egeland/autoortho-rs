//! SimHeaven X-World compatibility management.
//!
//! When SimHeaven compatibility is enabled, this module manages `scenery_packs.ini`
//! to disable AutoOrtho's road/label overlays (`yAutoOrtho_Overlays`) while keeping
//! ortho imagery packs (`z_ao_*`) enabled.
//!
//! This allows SimHeaven's roads/labels to show through while retaining AutoOrtho
//! ortho imagery as a fallback in regions where SimHeaven isn't installed.

use std::path::Path;

use thiserror::Error;

use super::packs_ini::{IniError, read_packs_ini, write_packs_ini};

#[derive(Debug, Error)]
pub enum SimHeavenError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("scenery_packs.ini error: {0}")]
    Ini(#[from] IniError),
    #[error("scenery_packs.ini not found")]
    IniNotFound,
    #[error("SimHeaven packages missing for regions: {0:?}")]
    MissingPackages(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum KubilusRegion {
    NorthAmerica,
    SouthAmerica,
    Europe,
    Asia,
    Africa,
    AustraliaPacific,
}

impl KubilusRegion {
    pub fn as_str(&self) -> &'static str {
        match self {
            KubilusRegion::NorthAmerica => "na",
            KubilusRegion::SouthAmerica => "sa",
            KubilusRegion::Europe => "eur",
            KubilusRegion::Asia => "asi",
            KubilusRegion::Africa => "afr",
            KubilusRegion::AustraliaPacific => "aus_pac",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "na" => Some(KubilusRegion::NorthAmerica),
            "sa" => Some(KubilusRegion::SouthAmerica),
            "eur" => Some(KubilusRegion::Europe),
            "asi" => Some(KubilusRegion::Asia),
            "afr" => Some(KubilusRegion::Africa),
            "aus_pac" => Some(KubilusRegion::AustraliaPacific),
            _ => None,
        }
    }

    pub fn to_simheaven_region(&self) -> &'static str {
        match self {
            KubilusRegion::NorthAmerica => "America",
            KubilusRegion::SouthAmerica => "America",
            KubilusRegion::Europe => "Europe",
            KubilusRegion::Asia => "Asia",
            KubilusRegion::Africa => "Africa",
            KubilusRegion::AustraliaPacific => "Australia-Oceania",
        }
    }
}

pub struct SimHeavenCheckResult {
    pub all_present: bool,
    pub missing_regions: Vec<String>,
}

const OVERLAYS_PACK: &str = "Custom Scenery/yAutoOrtho_Overlays/";

pub fn check_simheaven_packages(
    xplane_dir: &Path,
    active_regions: &[String],
) -> Result<SimHeavenCheckResult, SimHeavenError> {
    let entries = read_packs_ini(xplane_dir).map_err(|_| SimHeavenError::IniNotFound)?;

    let mut missing = Vec::new();

    for region_id in active_regions {
        let kubilus = match KubilusRegion::from_str(region_id) {
            Some(r) => r,
            None => continue,
        };
        let simheaven_region = kubilus.to_simheaven_region();

        let xp11_pattern = format!("Custom Scenery/simHeaven_X-{}", simheaven_region);
        let xp12_pattern = format!("Custom Scenery/simHeaven_X-World_{}", simheaven_region);

        let found = entries.iter().any(|e| {
            e.enabled && (e.path.contains(&xp11_pattern) || e.path.contains(&xp12_pattern))
        });

        if !found {
            missing.push(region_id.clone());
        }
    }

    Ok(SimHeavenCheckResult {
        all_present: missing.is_empty(),
        missing_regions: missing,
    })
}

pub fn apply_simheaven_compat(
    xplane_dir: &Path,
    enabled: bool,
    active_regions: &[String],
) -> Result<(), SimHeavenError> {
    if enabled {
        let check = check_simheaven_packages(xplane_dir, active_regions)?;
        if !check.all_present {
            return Err(SimHeavenError::MissingPackages(check.missing_regions));
        }
    }

    let entries = read_packs_ini(xplane_dir).map_err(|_| SimHeavenError::IniNotFound)?;

    let mut modified_entries = entries;
    let mut modified = false;

    for entry in &mut modified_entries {
        if entry.path.starts_with(OVERLAYS_PACK)
            || entry.path == OVERLAYS_PACK.trim_end_matches('/')
        {
            if enabled {
                if entry.enabled {
                    entry.enabled = false;
                    modified = true;
                    log::info!("Disabled AutoOrtho overlays for SimHeaven compatibility");
                }
            } else if !entry.enabled {
                entry.enabled = true;
                modified = true;
                log::info!("Enabled AutoOrtho overlays");
            }
        }
    }

    if modified {
        write_packs_ini(xplane_dir, &modified_entries)?;
    } else {
        log::info!("No AutoOrtho overlay entry found in scenery_packs.ini - skipping");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_ini(tmp: &TempDir, content: &str) {
        let custom_scenery = tmp.path().join("Custom Scenery");
        std::fs::create_dir_all(&custom_scenery).unwrap();
        std::fs::write(custom_scenery.join("scenery_packs.ini"), content).unwrap();
    }

    #[test]
    fn test_kubilus_region_mapping() {
        assert_eq!(
            KubilusRegion::from_str("na"),
            Some(KubilusRegion::NorthAmerica)
        );
        assert_eq!(
            KubilusRegion::from_str("sa"),
            Some(KubilusRegion::SouthAmerica)
        );
        assert_eq!(KubilusRegion::from_str("eur"), Some(KubilusRegion::Europe));
        assert_eq!(KubilusRegion::from_str("asi"), Some(KubilusRegion::Asia));
        assert_eq!(KubilusRegion::from_str("afr"), Some(KubilusRegion::Africa));
        assert_eq!(
            KubilusRegion::from_str("aus_pac"),
            Some(KubilusRegion::AustraliaPacific)
        );
        assert_eq!(KubilusRegion::from_str("invalid"), None);
    }

    #[test]
    fn test_simheaven_region_mapping() {
        assert_eq!(KubilusRegion::NorthAmerica.to_simheaven_region(), "America");
        assert_eq!(KubilusRegion::SouthAmerica.to_simheaven_region(), "America");
        assert_eq!(KubilusRegion::Europe.to_simheaven_region(), "Europe");
        assert_eq!(KubilusRegion::Asia.to_simheaven_region(), "Asia");
        assert_eq!(KubilusRegion::Africa.to_simheaven_region(), "Africa");
        assert_eq!(
            KubilusRegion::AustraliaPacific.to_simheaven_region(),
            "Australia-Oceania"
        );
    }

    #[test]
    fn test_check_simheaven_packages_xp12() {
        let tmp = TempDir::new().unwrap();
        setup_ini(
            &tmp,
            "I\n1000 Version\nSCENERY\n\n\
             SCENERY_PACK Custom Scenery/simHeaven_X-World_America/\n\
             SCENERY_PACK Custom Scenery/simHeaven_X-World_Europe/\n",
        );

        let result =
            check_simheaven_packages(tmp.path(), &["na".to_string(), "eur".to_string()]).unwrap();
        assert!(result.all_present);
        assert!(result.missing_regions.is_empty());
    }

    #[test]
    fn test_check_simheaven_packages_xp11() {
        let tmp = TempDir::new().unwrap();
        setup_ini(
            &tmp,
            "I\n1000 Version\nSCENERY\n\n\
             SCENERY_PACK Custom Scenery/simHeaven_X-America/\n\
             SCENERY_PACK Custom Scenery/simHeaven_X-Europe/\n",
        );

        let result =
            check_simheaven_packages(tmp.path(), &["na".to_string(), "eur".to_string()]).unwrap();
        assert!(result.all_present);
    }

    #[test]
    fn test_check_simheaven_packages_missing() {
        let tmp = TempDir::new().unwrap();
        setup_ini(
            &tmp,
            "I\n1000 Version\nSCENERY\n\n\
             SCENERY_PACK Custom Scenery/simHeaven_X-America/\n",
        );

        let result =
            check_simheaven_packages(tmp.path(), &["na".to_string(), "eur".to_string()]).unwrap();
        assert!(!result.all_present);
        assert_eq!(result.missing_regions, vec!["eur".to_string()]);
    }

    #[test]
    fn test_apply_simheaven_compat_disable_overlays() {
        let tmp = TempDir::new().unwrap();
        setup_ini(
            &tmp,
            "I\n1000 Version\nSCENERY\n\n\
             SCENERY_PACK Custom Scenery/yAutoOrtho_Overlays/\n\
             SCENERY_PACK Custom Scenery/simHeaven_X-World_America/\n",
        );

        apply_simheaven_compat(tmp.path(), true, &["na".to_string()]).unwrap();

        let entries = read_packs_ini(tmp.path()).unwrap();
        let overlay = entries
            .iter()
            .find(|e| e.path.contains("yAutoOrtho_Overlays"))
            .unwrap();
        assert!(!overlay.enabled);
    }

    #[test]
    fn test_apply_simheaven_compat_enable_overlays() {
        let tmp = TempDir::new().unwrap();
        setup_ini(
            &tmp,
            "I\n1000 Version\nSCENERY\n\n\
             SCENERY_PACK_DISABLED Custom Scenery/yAutoOrtho_Overlays/\n",
        );

        apply_simheaven_compat(tmp.path(), false, &["na".to_string()]).unwrap();

        let entries = read_packs_ini(tmp.path()).unwrap();
        let overlay = entries
            .iter()
            .find(|e| e.path.contains("yAutoOrtho_Overlays"))
            .unwrap();
        assert!(overlay.enabled);
    }

    #[test]
    fn test_apply_simheaven_compat_missing_packages() {
        let tmp = TempDir::new().unwrap();
        setup_ini(
            &tmp,
            "I\n1000 Version\nSCENERY\n\n\
             SCENERY_PACK Custom Scenery/yAutoOrtho_Overlays/\n",
        );

        let result = apply_simheaven_compat(tmp.path(), true, &["na".to_string()]);
        assert!(result.is_err());
        assert!(matches!(result, Err(SimHeavenError::MissingPackages(_))));
    }

    #[test]
    fn test_ini_not_found() {
        let tmp = TempDir::new().unwrap();

        let result = check_simheaven_packages(tmp.path(), &["na".to_string()]);
        assert!(result.is_err());
    }
}
