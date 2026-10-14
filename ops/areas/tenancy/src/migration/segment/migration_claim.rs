use super::*;

use acme_kit::text;

/// Where a migration claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationClaimStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl MigrationClaimStatus {
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

impl std::fmt::Display for MigrationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration claim as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationClaim {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub port: i64,
    pub notes: String,
    pub threshold: f64,
    pub attempts: i64,
    pub external_ref: String,
    pub status: MigrationClaimStatus,
}

impl MigrationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationClaimStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationClaimStatus) -> bool {
        match self.status {
            MigrationClaimStatus::New => {
                match next {
                    MigrationClaimStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationClaimStatus::Queued => {
                return next == MigrationClaimStatus::Running || next == MigrationClaimStatus::Done;
            }
            MigrationClaimStatus::Running => {
                match next {
                    MigrationClaimStatus::Done | MigrationClaimStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationClaimStatus::Done => {
                match next {
                    MigrationClaimStatus::Failed => {
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

    pub fn move_to(&mut self, next: MigrationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "migration claim", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        (self.threshold * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationClaim {
        MigrationClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            port: self.port,
            notes: self.notes.clone(),
            threshold: self.threshold,
            attempts: self.attempts,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a migration claim in status new.
pub fn new_migration_claim(code: &str, title: &str) -> MigrationClaim {
    MigrationClaim {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        port: 0,
        notes: "".to_string(),
        threshold: 0.0,
        attempts: 0,
        external_ref: "".to_string(),
        status: MigrationClaimStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_migration_claim("code-637", "title-181");
        assert_eq!(item.status, MigrationClaimStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_migration_claim("code-637", "title-181");
        if item.move_to(MigrationClaimStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationClaimStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_migration_claim("code-637", "title-181");
        item.status = MigrationClaimStatus::Failed;
        assert!(
            item.move_to(MigrationClaimStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
