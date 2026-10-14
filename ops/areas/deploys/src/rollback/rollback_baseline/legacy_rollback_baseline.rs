use super::*;

use acme_kit::text;

/// Where a legacy rollback baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRollbackBaselineStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyRollbackBaselineStatus {
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

impl std::fmt::Display for LegacyRollbackBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy rollback baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackBaseline {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub active: bool,
    pub timezone: String,
    pub maximum: f64,
    pub country: String,
    pub status: LegacyRollbackBaselineStatus,
}

impl LegacyRollbackBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRollbackBaselineStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRollbackBaselineStatus) -> bool {
        match self.status {
            LegacyRollbackBaselineStatus::New => {
                return next == LegacyRollbackBaselineStatus::Failed;
            }
            LegacyRollbackBaselineStatus::Queued => {
                return next == LegacyRollbackBaselineStatus::Done || next == LegacyRollbackBaselineStatus::Failed;
            }
            LegacyRollbackBaselineStatus::Running => {
                return next == LegacyRollbackBaselineStatus::Done || next == LegacyRollbackBaselineStatus::Failed;
            }
            LegacyRollbackBaselineStatus::Done => {
                match next {
                    LegacyRollbackBaselineStatus::Failed => {
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

    pub fn move_to(&mut self, next: LegacyRollbackBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy rollback baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy rollback baseline", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.updated_at as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRollbackBaseline {
        LegacyRollbackBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            active: self.active,
            timezone: self.timezone.clone(),
            maximum: self.maximum,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a legacy rollback baseline in status new.
pub fn new_legacy_rollback_baseline(code: &str, title: &str) -> LegacyRollbackBaseline {
    LegacyRollbackBaseline {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        active: false,
        timezone: "".to_string(),
        maximum: 0.0,
        country: "".to_string(),
        status: LegacyRollbackBaselineStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_rollback_baseline("code-732", "title-18");
        assert_eq!(item.status, LegacyRollbackBaselineStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_legacy_rollback_baseline("code-732", "title-18");
        if item.move_to(LegacyRollbackBaselineStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRollbackBaselineStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_rollback_baseline("code-732", "title-18");
        item.status = LegacyRollbackBaselineStatus::Failed;
        assert!(
            item.move_to(LegacyRollbackBaselineStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
