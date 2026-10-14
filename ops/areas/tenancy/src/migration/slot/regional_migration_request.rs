use super::*;

use acme_kit::text;

/// Where a regional migration request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalMigrationRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalMigrationRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RegionalMigrationRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional migration request as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationRequest {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub issuer: String,
    pub cluster: String,
    pub duration_minutes: i64,
    pub floor_value: f64,
    pub audience: String,
    pub status: RegionalMigrationRequestStatus,
}

impl RegionalMigrationRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalMigrationRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalMigrationRequestStatus) -> bool {
        match self.status {
            RegionalMigrationRequestStatus::New => {
                return next == RegionalMigrationRequestStatus::Running || next == RegionalMigrationRequestStatus::Done;
            }
            RegionalMigrationRequestStatus::Queued => {
                match next {
                    RegionalMigrationRequestStatus::Running | RegionalMigrationRequestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalMigrationRequestStatus::Running => {
                return next == RegionalMigrationRequestStatus::Done || next == RegionalMigrationRequestStatus::Failed;
            }
            RegionalMigrationRequestStatus::Done => {
                match next {
                    RegionalMigrationRequestStatus::Failed => {
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

    pub fn move_to(&mut self, next: RegionalMigrationRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional migration request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        (self.floor_value * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalMigrationRequest {
        RegionalMigrationRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            issuer: self.issuer.clone(),
            cluster: self.cluster.clone(),
            duration_minutes: self.duration_minutes,
            floor_value: self.floor_value,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a regional migration request in status new.
pub fn new_regional_migration_request(code: &str, name: &str) -> RegionalMigrationRequest {
    RegionalMigrationRequest {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        issuer: "".to_string(),
        cluster: "".to_string(),
        duration_minutes: 0,
        floor_value: 0.0,
        audience: "".to_string(),
        status: RegionalMigrationRequestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_migration_request("code-324", "name-836");
        assert_eq!(item.status, RegionalMigrationRequestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_regional_migration_request("code-324", "name-836");
        if item.move_to(RegionalMigrationRequestStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalMigrationRequestStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_migration_request("code-324", "name-836");
        item.status = RegionalMigrationRequestStatus::Failed;
        assert!(
            item.move_to(RegionalMigrationRequestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
