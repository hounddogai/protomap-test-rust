use super::*;

use acme_kit::text;

/// Where a weighted limit audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedLimitAuditStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedLimitAuditStatus {
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

impl std::fmt::Display for WeightedLimitAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted limit audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitAudit {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub updated_at: i64,
    pub archived: bool,
    pub region: String,
    pub status: WeightedLimitAuditStatus,
}

impl WeightedLimitAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedLimitAuditStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedLimitAuditStatus) -> bool {
        match self.status {
            WeightedLimitAuditStatus::Planned => {
                match next {
                    WeightedLimitAuditStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedLimitAuditStatus::Scheduled => {
                return next == WeightedLimitAuditStatus::InProgress;
            }
            WeightedLimitAuditStatus::InProgress => {
                return next == WeightedLimitAuditStatus::Cancelled;
            }
            WeightedLimitAuditStatus::Completed => {
                match next {
                    WeightedLimitAuditStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: WeightedLimitAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted limit audit {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "weighted limit audit", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> WeightedLimitAudit {
        WeightedLimitAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            updated_at: self.updated_at,
            archived: self.archived,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a weighted limit audit in status planned.
pub fn new_weighted_limit_audit(code: &str, label: &str) -> WeightedLimitAudit {
    WeightedLimitAudit {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        updated_at: 0,
        archived: false,
        region: "".to_string(),
        status: WeightedLimitAuditStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_limit_audit("code-125", "label-834");
        assert_eq!(item.status, WeightedLimitAuditStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_weighted_limit_audit("code-125", "label-834");
        if item.move_to(WeightedLimitAuditStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedLimitAuditStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_limit_audit("code-125", "label-834");
        item.status = WeightedLimitAuditStatus::Cancelled;
        assert!(
            item.move_to(WeightedLimitAuditStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
