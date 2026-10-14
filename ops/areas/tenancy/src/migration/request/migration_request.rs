use super::*;

use acme_kit::text;

/// Where a migration request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl MigrationRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for MigrationRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration request as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRequest {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub weight: f64,
    pub cost: f64,
    pub segment: String,
    pub quota_units: i64,
    pub country: String,
    pub cluster: String,
    pub version: i64,
    pub status: MigrationRequestStatus,
}

impl MigrationRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationRequestStatus) -> bool {
        match self.status {
            MigrationRequestStatus::Draft => {
                match next {
                    MigrationRequestStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationRequestStatus::Pending => {
                return next == MigrationRequestStatus::Active || next == MigrationRequestStatus::Closed;
            }
            MigrationRequestStatus::Active => {
                return next == MigrationRequestStatus::Suspended || next == MigrationRequestStatus::Closed;
            }
            MigrationRequestStatus::Suspended => {
                return next == MigrationRequestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "migration request", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.cost - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationRequest {
        MigrationRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            weight: self.weight,
            cost: self.cost,
            segment: self.segment.clone(),
            quota_units: self.quota_units,
            country: self.country.clone(),
            cluster: self.cluster.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a migration request in status draft.
pub fn new_migration_request(code: &str, name: &str) -> MigrationRequest {
    MigrationRequest {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        weight: 0.0,
        cost: 0.0,
        segment: "".to_string(),
        quota_units: 0,
        country: "".to_string(),
        cluster: "".to_string(),
        version: 0,
        status: MigrationRequestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_migration_request("code-804", "name-289");
        assert_eq!(item.status, MigrationRequestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_migration_request("code-804", "name-289");
        if item.move_to(MigrationRequestStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationRequestStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_migration_request("code-804", "name-289");
        item.status = MigrationRequestStatus::Closed;
        assert!(
            item.move_to(MigrationRequestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
