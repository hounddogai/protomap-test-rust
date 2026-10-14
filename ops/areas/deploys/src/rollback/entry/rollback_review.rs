use super::*;

use acme_kit::text;

/// Where a rollback review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollbackReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollbackReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback review as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackReview {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub burst: i64,
    pub batch_size: i64,
    pub starts_at: i64,
    pub scope: String,
    pub window_days: i64,
    pub status: RollbackReviewStatus,
}

impl RollbackReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackReviewStatus) -> bool {
        match self.status {
            RollbackReviewStatus::Created => {
                match next {
                    RollbackReviewStatus::Validated | RollbackReviewStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackReviewStatus::Validated => {
                return next == RollbackReviewStatus::Retired;
            }
            RollbackReviewStatus::Published => {
                return next == RollbackReviewStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollback review", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.burst as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackReview {
        RollbackReview {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            burst: self.burst,
            batch_size: self.batch_size,
            starts_at: self.starts_at,
            scope: self.scope.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a rollback review in status created.
pub fn new_rollback_review(code: &str, name: &str) -> RollbackReview {
    RollbackReview {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        burst: 0,
        batch_size: 0,
        starts_at: 0,
        scope: "".to_string(),
        window_days: 0,
        status: RollbackReviewStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rollback_review("code-493", "name-724");
        assert_eq!(item.status, RollbackReviewStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_rollback_review("code-493", "name-724");
        if item.move_to(RollbackReviewStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackReviewStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rollback_review("code-493", "name-724");
        item.status = RollbackReviewStatus::Retired;
        assert!(
            item.move_to(RollbackReviewStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
