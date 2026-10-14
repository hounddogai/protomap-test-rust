use super::*;

use acme_kit::text;

/// Where a alert allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertAllocationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AlertAllocationStatus {
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

impl std::fmt::Display for AlertAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One alert allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertAllocation {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub archived: bool,
    pub expires_at: i64,
    pub burst: i64,
    pub rate: f64,
    pub status: AlertAllocationStatus,
}

impl AlertAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertAllocationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertAllocationStatus) -> bool {
        match self.status {
            AlertAllocationStatus::Open => {
                return next == AlertAllocationStatus::Reopened;
            }
            AlertAllocationStatus::InReview => {
                return next == AlertAllocationStatus::Resolved;
            }
            AlertAllocationStatus::Resolved => {
                return next == AlertAllocationStatus::Reopened || next == AlertAllocationStatus::Archived;
            }
            AlertAllocationStatus::Reopened => {
                return next == AlertAllocationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert allocation {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.budget - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertAllocation {
        AlertAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            archived: self.archived,
            expires_at: self.expires_at,
            burst: self.burst,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a alert allocation in status open.
pub fn new_alert_allocation(code: &str, label: &str) -> AlertAllocation {
    AlertAllocation {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        archived: false,
        expires_at: 0,
        burst: 0,
        rate: 0.0,
        status: AlertAllocationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_alert_allocation("code-316", "label-946");
        assert_eq!(item.status, AlertAllocationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_alert_allocation("code-316", "label-946");
        if item.move_to(AlertAllocationStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertAllocationStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_alert_allocation("code-316", "label-946");
        item.status = AlertAllocationStatus::Archived;
        assert!(
            item.move_to(AlertAllocationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
