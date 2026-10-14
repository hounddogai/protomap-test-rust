use super::*;

use acme_kit::text;

/// Where a pending release rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingReleaseRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingReleaseRuleStatus {
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

impl std::fmt::Display for PendingReleaseRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending release rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseRule {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub contact_email: String,
    pub phone: String,
    pub pinned: bool,
    pub amount: f64,
    pub status: PendingReleaseRuleStatus,
}

impl PendingReleaseRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingReleaseRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingReleaseRuleStatus) -> bool {
        match self.status {
            PendingReleaseRuleStatus::Planned => {
                match next {
                    PendingReleaseRuleStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingReleaseRuleStatus::Scheduled => {
                return next == PendingReleaseRuleStatus::Completed;
            }
            PendingReleaseRuleStatus::InProgress => {
                match next {
                    PendingReleaseRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingReleaseRuleStatus::Completed => {
                match next {
                    PendingReleaseRuleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PendingReleaseRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending release rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.amount + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingReleaseRule {
        PendingReleaseRule {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            contact_email: self.contact_email.clone(),
            phone: self.phone.clone(),
            pinned: self.pinned,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending release rule in status planned.
pub fn new_pending_release_rule(code: &str, label: &str) -> PendingReleaseRule {
    PendingReleaseRule {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        contact_email: "".to_string(),
        phone: "".to_string(),
        pinned: false,
        amount: 0.0,
        status: PendingReleaseRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_pending_release_rule("code-319", "label-395");
        assert_eq!(item.status, PendingReleaseRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_pending_release_rule("code-319", "label-395");
        if item.move_to(PendingReleaseRuleStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingReleaseRuleStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_pending_release_rule("code-319", "label-395");
        item.status = PendingReleaseRuleStatus::Cancelled;
        assert!(
            item.move_to(PendingReleaseRuleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
