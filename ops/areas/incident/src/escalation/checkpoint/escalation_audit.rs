use super::*;

use acme_kit::text;

/// Where a escalation audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationAuditStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EscalationAuditStatus {
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

impl std::fmt::Display for EscalationAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a escalation audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAudit {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub created_at: i64,
    pub capacity: i64,
    pub rate: f64,
    pub checksum: String,
    pub cost: f64,
    pub category: String,
    pub status: EscalationAuditStatus,
}

impl EscalationAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationAuditStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationAuditStatus) -> bool {
        match self.status {
            EscalationAuditStatus::Planned => {
                match next {
                    EscalationAuditStatus::Completed | EscalationAuditStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationAuditStatus::Scheduled => {
                return next == EscalationAuditStatus::Completed || next == EscalationAuditStatus::Cancelled;
            }
            EscalationAuditStatus::InProgress => {
                match next {
                    EscalationAuditStatus::Completed | EscalationAuditStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationAuditStatus::Completed => {
                return next == EscalationAuditStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation audit {} cannot move from {} to {}",
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
        format!("{} {}", "escalation audit", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.expires_at as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationAudit {
        EscalationAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            created_at: self.created_at,
            capacity: self.capacity,
            rate: self.rate,
            checksum: self.checksum.clone(),
            cost: self.cost,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a escalation audit in status planned.
pub fn new_escalation_audit(code: &str, title: &str) -> EscalationAudit {
    EscalationAudit {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        created_at: 0,
        capacity: 0,
        rate: 0.0,
        checksum: "".to_string(),
        cost: 0.0,
        category: "".to_string(),
        status: EscalationAuditStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_escalation_audit("code-437", "title-373");
        assert_eq!(item.status, EscalationAuditStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_escalation_audit("code-437", "title-373");
        if item.move_to(EscalationAuditStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationAuditStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_escalation_audit("code-437", "title-373");
        item.status = EscalationAuditStatus::Cancelled;
        assert!(
            item.move_to(EscalationAuditStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
