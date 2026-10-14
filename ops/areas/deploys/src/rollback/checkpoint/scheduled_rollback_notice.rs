use super::*;

use acme_kit::text;

/// Where a scheduled rollback notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRollbackNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledRollbackNoticeStatus {
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

impl std::fmt::Display for ScheduledRollbackNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled rollback notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRollbackNotice {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub notes: String,
    pub discount: f64,
    pub currency: String,
    pub quantity: i64,
    pub status: ScheduledRollbackNoticeStatus,
}

impl ScheduledRollbackNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRollbackNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRollbackNoticeStatus) -> bool {
        match self.status {
            ScheduledRollbackNoticeStatus::New => {
                return next == ScheduledRollbackNoticeStatus::Failed;
            }
            ScheduledRollbackNoticeStatus::Queued => {
                return next == ScheduledRollbackNoticeStatus::Done;
            }
            ScheduledRollbackNoticeStatus::Running => {
                match next {
                    ScheduledRollbackNoticeStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRollbackNoticeStatus::Done => {
                match next {
                    ScheduledRollbackNoticeStatus::Failed => {
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

    pub fn move_to(&mut self, next: ScheduledRollbackNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled rollback notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.quantity as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledRollbackNotice {
        ScheduledRollbackNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            notes: self.notes.clone(),
            discount: self.discount,
            currency: self.currency.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled rollback notice in status new.
pub fn new_scheduled_rollback_notice(code: &str, label: &str) -> ScheduledRollbackNotice {
    ScheduledRollbackNotice {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        notes: "".to_string(),
        discount: 0.0,
        currency: "".to_string(),
        quantity: 0,
        status: ScheduledRollbackNoticeStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_rollback_notice("code-295", "label-254");
        assert_eq!(item.status, ScheduledRollbackNoticeStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_scheduled_rollback_notice("code-295", "label-254");
        if item.move_to(ScheduledRollbackNoticeStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledRollbackNoticeStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_rollback_notice("code-295", "label-254");
        item.status = ScheduledRollbackNoticeStatus::Failed;
        assert!(
            item.move_to(ScheduledRollbackNoticeStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
