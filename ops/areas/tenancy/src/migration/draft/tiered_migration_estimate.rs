use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a tiered migration estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredMigrationEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TieredMigrationEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TieredMigrationEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered migration estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationEstimate {
    pub code: String,
    pub name: String,
    pub category: String,
    pub attributes: BTreeMap<String, String>,
    pub retries: i64,
    pub status: TieredMigrationEstimateStatus,
}

impl TieredMigrationEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredMigrationEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredMigrationEstimateStatus) -> bool {
        match self.status {
            TieredMigrationEstimateStatus::Requested => {
                match next {
                    TieredMigrationEstimateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredMigrationEstimateStatus::Granted => {
                match next {
                    TieredMigrationEstimateStatus::Revoked | TieredMigrationEstimateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredMigrationEstimateStatus::Revoked => {
                return next == TieredMigrationEstimateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredMigrationEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered migration estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered migration estimate", self.code)
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

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> TieredMigrationEstimate {
        TieredMigrationEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            attributes: self.attributes.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tiered migration estimate in status requested.
pub fn new_tiered_migration_estimate(code: &str, name: &str) -> TieredMigrationEstimate {
    TieredMigrationEstimate {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        attributes: BTreeMap::new(),
        retries: 0,
        status: TieredMigrationEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_tiered_migration_estimate("code-202", "name-952");
        assert_eq!(item.status, TieredMigrationEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_tiered_migration_estimate("code-202", "name-952");
        if item.move_to(TieredMigrationEstimateStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredMigrationEstimateStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_tiered_migration_estimate("code-202", "name-952");
        item.status = TieredMigrationEstimateStatus::Expired;
        assert!(
            item.move_to(TieredMigrationEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
