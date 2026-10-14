use super::*;

use acme_kit::text;

/// Where a regional zone manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalZoneManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalZoneManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalZoneManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional zone manifest as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneManifest {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub retries: i64,
    pub currency: String,
    pub status: RegionalZoneManifestStatus,
}

impl RegionalZoneManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalZoneManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalZoneManifestStatus) -> bool {
        match self.status {
            RegionalZoneManifestStatus::Created => {
                match next {
                    RegionalZoneManifestStatus::Validated | RegionalZoneManifestStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalZoneManifestStatus::Validated => {
                match next {
                    RegionalZoneManifestStatus::Published | RegionalZoneManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalZoneManifestStatus::Published => {
                match next {
                    RegionalZoneManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalZoneManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional zone manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.retries as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalZoneManifest {
        RegionalZoneManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            retries: self.retries,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional zone manifest in status created.
pub fn new_regional_zone_manifest(code: &str, title: &str) -> RegionalZoneManifest {
    RegionalZoneManifest {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        retries: 0,
        currency: "".to_string(),
        status: RegionalZoneManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_regional_zone_manifest("code-253", "title-864");
        assert_eq!(item.status, RegionalZoneManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_regional_zone_manifest("code-253", "title-864");
        if item.move_to(RegionalZoneManifestStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalZoneManifestStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_regional_zone_manifest("code-253", "title-864");
        item.status = RegionalZoneManifestStatus::Retired;
        assert!(
            item.move_to(RegionalZoneManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
