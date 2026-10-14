use super::*;

use acme_kit::text;

/// Where a migration preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationPreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl MigrationPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for MigrationPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One migration preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationPreference {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub created_at: i64,
    pub owner_id: String,
    pub status: MigrationPreferenceStatus,
}

impl MigrationPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationPreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationPreferenceStatus) -> bool {
        match self.status {
            MigrationPreferenceStatus::Requested => {
                match next {
                    MigrationPreferenceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationPreferenceStatus::Granted => {
                return next == MigrationPreferenceStatus::Revoked;
            }
            MigrationPreferenceStatus::Revoked => {
                return next == MigrationPreferenceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> MigrationPreference {
        MigrationPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            created_at: self.created_at,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a migration preference in status requested.
pub fn new_migration_preference(code: &str, name: &str) -> MigrationPreference {
    MigrationPreference {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        created_at: 0,
        owner_id: "".to_string(),
        status: MigrationPreferenceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_migration_preference("code-573", "name-199");
        assert_eq!(item.status, MigrationPreferenceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_migration_preference("code-573", "name-199");
        if item.move_to(MigrationPreferenceStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationPreferenceStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_migration_preference("code-573", "name-199");
        item.status = MigrationPreferenceStatus::Expired;
        assert!(
            item.move_to(MigrationPreferenceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
