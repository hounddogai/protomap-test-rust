use super::*;

use acme_kit::text;

/// Where a seasonal migration score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalMigrationScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalMigrationScoreStatus {
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

impl std::fmt::Display for SeasonalMigrationScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal migration score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMigrationScore {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub threshold: f64,
    pub expires_at: i64,
    pub status: SeasonalMigrationScoreStatus,
}

impl SeasonalMigrationScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalMigrationScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalMigrationScoreStatus) -> bool {
        match self.status {
            SeasonalMigrationScoreStatus::New => {
                return next == SeasonalMigrationScoreStatus::Queued || next == SeasonalMigrationScoreStatus::Done;
            }
            SeasonalMigrationScoreStatus::Queued => {
                return next == SeasonalMigrationScoreStatus::Running || next == SeasonalMigrationScoreStatus::Done;
            }
            SeasonalMigrationScoreStatus::Running => {
                return next == SeasonalMigrationScoreStatus::Done || next == SeasonalMigrationScoreStatus::Failed;
            }
            SeasonalMigrationScoreStatus::Done => {
                match next {
                    SeasonalMigrationScoreStatus::Failed => {
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

    pub fn move_to(&mut self, next: SeasonalMigrationScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal migration score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal migration score", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.expires_at as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalMigrationScore {
        SeasonalMigrationScore {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            threshold: self.threshold,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal migration score in status new.
pub fn new_seasonal_migration_score(code: &str, title: &str) -> SeasonalMigrationScore {
    SeasonalMigrationScore {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        threshold: 0.0,
        expires_at: 0,
        status: SeasonalMigrationScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_seasonal_migration_score("code-504", "title-673");
        assert_eq!(item.status, SeasonalMigrationScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_seasonal_migration_score("code-504", "title-673");
        if item.move_to(SeasonalMigrationScoreStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalMigrationScoreStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_seasonal_migration_score("code-504", "title-673");
        item.status = SeasonalMigrationScoreStatus::Failed;
        assert!(
            item.move_to(SeasonalMigrationScoreStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
