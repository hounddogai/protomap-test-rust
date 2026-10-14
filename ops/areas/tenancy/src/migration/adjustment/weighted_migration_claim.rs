use super::*;

use acme_kit::text;

/// Where a weighted migration claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedMigrationClaimStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedMigrationClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedMigrationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted migration claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMigrationClaim {
    pub code: String,
    pub name: String,
    pub category: String,
    pub country: String,
    pub channel: String,
    pub version: i64,
    pub attempts: i64,
    pub capacity: i64,
    pub description: String,
    pub tenant_id: String,
    pub status: WeightedMigrationClaimStatus,
}

impl WeightedMigrationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedMigrationClaimStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedMigrationClaimStatus) -> bool {
        match self.status {
            WeightedMigrationClaimStatus::Created => {
                match next {
                    WeightedMigrationClaimStatus::Validated | WeightedMigrationClaimStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedMigrationClaimStatus::Validated => {
                match next {
                    WeightedMigrationClaimStatus::Published | WeightedMigrationClaimStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedMigrationClaimStatus::Published => {
                return next == WeightedMigrationClaimStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedMigrationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted migration claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.capacity as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedMigrationClaim {
        WeightedMigrationClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            country: self.country.clone(),
            channel: self.channel.clone(),
            version: self.version,
            attempts: self.attempts,
            capacity: self.capacity,
            description: self.description.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a weighted migration claim in status created.
pub fn new_weighted_migration_claim(code: &str, name: &str) -> WeightedMigrationClaim {
    WeightedMigrationClaim {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        country: "".to_string(),
        channel: "".to_string(),
        version: 0,
        attempts: 0,
        capacity: 0,
        description: "".to_string(),
        tenant_id: "".to_string(),
        status: WeightedMigrationClaimStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_migration_claim("code-701", "name-292");
        assert_eq!(item.status, WeightedMigrationClaimStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_weighted_migration_claim("code-701", "name-292");
        if item.move_to(WeightedMigrationClaimStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedMigrationClaimStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_migration_claim("code-701", "name-292");
        item.status = WeightedMigrationClaimStatus::Retired;
        assert!(
            item.move_to(WeightedMigrationClaimStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
