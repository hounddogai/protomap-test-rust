use super::*;

use acme_kit::text;

/// Where a adaptive migration checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveMigrationCheckpointStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveMigrationCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for AdaptiveMigrationCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive migration checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMigrationCheckpoint {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub ceiling: f64,
    pub ends_at: i64,
    pub cluster: String,
    pub fingerprint: String,
    pub host: String,
    pub status: AdaptiveMigrationCheckpointStatus,
}

impl AdaptiveMigrationCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveMigrationCheckpointStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveMigrationCheckpointStatus) -> bool {
        match self.status {
            AdaptiveMigrationCheckpointStatus::Open => {
                match next {
                    AdaptiveMigrationCheckpointStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveMigrationCheckpointStatus::InReview => {
                match next {
                    AdaptiveMigrationCheckpointStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveMigrationCheckpointStatus::Resolved => {
                match next {
                    AdaptiveMigrationCheckpointStatus::Reopened | AdaptiveMigrationCheckpointStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveMigrationCheckpointStatus::Reopened => {
                return next == AdaptiveMigrationCheckpointStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveMigrationCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive migration checkpoint {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        (self.ceiling * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveMigrationCheckpoint {
        AdaptiveMigrationCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            ceiling: self.ceiling,
            ends_at: self.ends_at,
            cluster: self.cluster.clone(),
            fingerprint: self.fingerprint.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a adaptive migration checkpoint in status open.
pub fn new_adaptive_migration_checkpoint(code: &str, title: &str) -> AdaptiveMigrationCheckpoint {
    AdaptiveMigrationCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        ceiling: 0.0,
        ends_at: 0,
        cluster: "".to_string(),
        fingerprint: "".to_string(),
        host: "".to_string(),
        status: AdaptiveMigrationCheckpointStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_migration_checkpoint("code-830", "title-558");
        assert_eq!(item.status, AdaptiveMigrationCheckpointStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_adaptive_migration_checkpoint("code-830", "title-558");
        if item.move_to(AdaptiveMigrationCheckpointStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveMigrationCheckpointStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_migration_checkpoint("code-830", "title-558");
        item.status = AdaptiveMigrationCheckpointStatus::Archived;
        assert!(
            item.move_to(AdaptiveMigrationCheckpointStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
