use super::*;

use acme_kit::text;

/// Where a scheduled rollback record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRollbackRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledRollbackRecordStatus {
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

impl std::fmt::Display for ScheduledRollbackRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled rollback record as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRollbackRecord {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub burst: i64,
    pub source: String,
    pub status: ScheduledRollbackRecordStatus,
}

impl ScheduledRollbackRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRollbackRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRollbackRecordStatus) -> bool {
        match self.status {
            ScheduledRollbackRecordStatus::Open => {
                return next == ScheduledRollbackRecordStatus::InReview;
            }
            ScheduledRollbackRecordStatus::InReview => {
                match next {
                    ScheduledRollbackRecordStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRollbackRecordStatus::Resolved => {
                return next == ScheduledRollbackRecordStatus::Reopened
                    || next == ScheduledRollbackRecordStatus::Archived;
            }
            ScheduledRollbackRecordStatus::Reopened => {
                return next == ScheduledRollbackRecordStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledRollbackRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled rollback record {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.burst as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledRollbackRecord {
        ScheduledRollbackRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            burst: self.burst,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a scheduled rollback record in status open.
pub fn new_scheduled_rollback_record(code: &str, label: &str) -> ScheduledRollbackRecord {
    ScheduledRollbackRecord {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        burst: 0,
        source: "".to_string(),
        status: ScheduledRollbackRecordStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_rollback_record("code-330", "label-285");
        assert_eq!(item.status, ScheduledRollbackRecordStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_scheduled_rollback_record("code-330", "label-285");
        if item.move_to(ScheduledRollbackRecordStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledRollbackRecordStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_rollback_record("code-330", "label-285");
        item.status = ScheduledRollbackRecordStatus::Archived;
        assert!(
            item.move_to(ScheduledRollbackRecordStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
