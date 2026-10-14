use super::*;

use acme_kit::text;

/// Where a legacy retention estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRetentionEstimateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyRetentionEstimateStatus {
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

impl std::fmt::Display for LegacyRetentionEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy retention estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionEstimate {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub limit: i64,
    pub threshold: f64,
    pub floor_value: f64,
    pub created_at: i64,
    pub margin: f64,
    pub budget: f64,
    pub grace_days: i64,
    pub status: LegacyRetentionEstimateStatus,
}

impl LegacyRetentionEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRetentionEstimateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRetentionEstimateStatus) -> bool {
        match self.status {
            LegacyRetentionEstimateStatus::New => {
                return next == LegacyRetentionEstimateStatus::Done;
            }
            LegacyRetentionEstimateStatus::Queued => {
                return next == LegacyRetentionEstimateStatus::Failed;
            }
            LegacyRetentionEstimateStatus::Running => {
                return next == LegacyRetentionEstimateStatus::Done || next == LegacyRetentionEstimateStatus::Failed;
            }
            LegacyRetentionEstimateStatus::Done => {
                match next {
                    LegacyRetentionEstimateStatus::Failed => {
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

    pub fn move_to(&mut self, next: LegacyRetentionEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy retention estimate {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.floor_value - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRetentionEstimate {
        LegacyRetentionEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            limit: self.limit,
            threshold: self.threshold,
            floor_value: self.floor_value,
            created_at: self.created_at,
            margin: self.margin,
            budget: self.budget,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy retention estimate in status new.
pub fn new_legacy_retention_estimate(code: &str, name: &str) -> LegacyRetentionEstimate {
    LegacyRetentionEstimate {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        limit: 0,
        threshold: 0.0,
        floor_value: 0.0,
        created_at: 0,
        margin: 0.0,
        budget: 0.0,
        grace_days: 0,
        status: LegacyRetentionEstimateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_retention_estimate("code-838", "name-885");
        assert_eq!(item.status, LegacyRetentionEstimateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_legacy_retention_estimate("code-838", "name-885");
        if item.move_to(LegacyRetentionEstimateStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRetentionEstimateStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_retention_estimate("code-838", "name-885");
        item.status = LegacyRetentionEstimateStatus::Failed;
        assert!(
            item.move_to(LegacyRetentionEstimateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
