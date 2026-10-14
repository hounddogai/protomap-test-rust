use super::*;

use acme_kit::text;

/// Where a tiered runbook audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRunbookAuditStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredRunbookAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TieredRunbookAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered runbook audit as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookAudit {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub category: String,
    pub channel: String,
    pub approved: bool,
    pub external_ref: String,
    pub duration_minutes: i64,
    pub status: TieredRunbookAuditStatus,
}

impl TieredRunbookAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRunbookAuditStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRunbookAuditStatus) -> bool {
        match self.status {
            TieredRunbookAuditStatus::Planned => {
                return next == TieredRunbookAuditStatus::Scheduled;
            }
            TieredRunbookAuditStatus::Scheduled => {
                return next == TieredRunbookAuditStatus::Completed || next == TieredRunbookAuditStatus::Cancelled;
            }
            TieredRunbookAuditStatus::InProgress => {
                return next == TieredRunbookAuditStatus::Cancelled;
            }
            TieredRunbookAuditStatus::Completed => {
                match next {
                    TieredRunbookAuditStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TieredRunbookAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered runbook audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.score - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRunbookAudit {
        TieredRunbookAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            category: self.category.clone(),
            channel: self.channel.clone(),
            approved: self.approved,
            external_ref: self.external_ref.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tiered runbook audit in status planned.
pub fn new_tiered_runbook_audit(code: &str, name: &str) -> TieredRunbookAudit {
    TieredRunbookAudit {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        category: "".to_string(),
        channel: "".to_string(),
        approved: false,
        external_ref: "".to_string(),
        duration_minutes: 0,
        status: TieredRunbookAuditStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_runbook_audit("code-682", "name-850");
        assert_eq!(item.status, TieredRunbookAuditStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_tiered_runbook_audit("code-682", "name-850");
        if item.move_to(TieredRunbookAuditStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredRunbookAuditStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_runbook_audit("code-682", "name-850");
        item.status = TieredRunbookAuditStatus::Cancelled;
        assert!(
            item.move_to(TieredRunbookAuditStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
