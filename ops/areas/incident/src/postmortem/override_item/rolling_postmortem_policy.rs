use super::*;

use acme_kit::text;

/// Where a rolling postmortem policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPostmortemPolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingPostmortemPolicyStatus {
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

impl std::fmt::Display for RollingPostmortemPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling postmortem policy as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPostmortemPolicy {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub budget: f64,
    pub pinned: bool,
    pub port: i64,
    pub discount: f64,
    pub owner_id: String,
    pub status: RollingPostmortemPolicyStatus,
}

impl RollingPostmortemPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPostmortemPolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPostmortemPolicyStatus) -> bool {
        match self.status {
            RollingPostmortemPolicyStatus::Planned => {
                match next {
                    RollingPostmortemPolicyStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPostmortemPolicyStatus::Scheduled => {
                match next {
                    RollingPostmortemPolicyStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPostmortemPolicyStatus::InProgress => {
                return next == RollingPostmortemPolicyStatus::Completed;
            }
            RollingPostmortemPolicyStatus::Completed => {
                return next == RollingPostmortemPolicyStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingPostmortemPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling postmortem policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling postmortem policy", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.budget + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPostmortemPolicy {
        RollingPostmortemPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            budget: self.budget,
            pinned: self.pinned,
            port: self.port,
            discount: self.discount,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling postmortem policy in status planned.
pub fn new_rolling_postmortem_policy(code: &str, label: &str) -> RollingPostmortemPolicy {
    RollingPostmortemPolicy {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        budget: 0.0,
        pinned: false,
        port: 0,
        discount: 0.0,
        owner_id: "".to_string(),
        status: RollingPostmortemPolicyStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_postmortem_policy("code-389", "label-598");
        assert_eq!(item.status, RollingPostmortemPolicyStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_rolling_postmortem_policy("code-389", "label-598");
        if item.move_to(RollingPostmortemPolicyStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPostmortemPolicyStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_postmortem_policy("code-389", "label-598");
        item.status = RollingPostmortemPolicyStatus::Cancelled;
        assert!(
            item.move_to(RollingPostmortemPolicyStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
