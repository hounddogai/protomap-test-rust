use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a regional credential rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalCredentialRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalCredentialRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalCredentialRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional credential rollup as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCredentialRollup {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub attributes: BTreeMap<String, String>,
    pub ends_at: i64,
    pub checksum: String,
    pub status: RegionalCredentialRollupStatus,
}

impl RegionalCredentialRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalCredentialRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalCredentialRollupStatus) -> bool {
        match self.status {
            RegionalCredentialRollupStatus::Created => {
                match next {
                    RegionalCredentialRollupStatus::Validated | RegionalCredentialRollupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalCredentialRollupStatus::Validated => {
                match next {
                    RegionalCredentialRollupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalCredentialRollupStatus::Published => {
                return next == RegionalCredentialRollupStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalCredentialRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional credential rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.rate + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalCredentialRollup {
        RegionalCredentialRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            attributes: self.attributes.clone(),
            ends_at: self.ends_at,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional credential rollup in status created.
pub fn new_regional_credential_rollup(code: &str, label: &str) -> RegionalCredentialRollup {
    RegionalCredentialRollup {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        attributes: BTreeMap::new(),
        ends_at: 0,
        checksum: "".to_string(),
        status: RegionalCredentialRollupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_regional_credential_rollup("code-381", "label-205");
        assert_eq!(item.status, RegionalCredentialRollupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_regional_credential_rollup("code-381", "label-205");
        if item.move_to(RegionalCredentialRollupStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalCredentialRollupStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_regional_credential_rollup("code-381", "label-205");
        item.status = RegionalCredentialRollupStatus::Retired;
        assert!(
            item.move_to(RegionalCredentialRollupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
