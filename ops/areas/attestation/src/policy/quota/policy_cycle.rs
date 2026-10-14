use super::*;

use acme_kit::text;

/// Where a policy cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyCycleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PolicyCycleStatus {
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

impl std::fmt::Display for PolicyCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One policy cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyCycle {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub ends_at: i64,
    pub timezone: String,
    pub retries: i64,
    pub contact_email: String,
    pub cost: f64,
    pub status: PolicyCycleStatus,
}

impl PolicyCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyCycleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyCycleStatus) -> bool {
        match self.status {
            PolicyCycleStatus::Planned => {
                match next {
                    PolicyCycleStatus::Scheduled | PolicyCycleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyCycleStatus::Scheduled => {
                match next {
                    PolicyCycleStatus::InProgress | PolicyCycleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyCycleStatus::InProgress => {
                match next {
                    PolicyCycleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyCycleStatus::Completed => {
                return next == PolicyCycleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.rotation_days as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyCycle {
        PolicyCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            ends_at: self.ends_at,
            timezone: self.timezone.clone(),
            retries: self.retries,
            contact_email: self.contact_email.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a policy cycle in status planned.
pub fn new_policy_cycle(code: &str, label: &str) -> PolicyCycle {
    PolicyCycle {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        ends_at: 0,
        timezone: "".to_string(),
        retries: 0,
        contact_email: "".to_string(),
        cost: 0.0,
        status: PolicyCycleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_policy_cycle("code-195", "label-316");
        assert_eq!(item.status, PolicyCycleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_policy_cycle("code-195", "label-316");
        if item.move_to(PolicyCycleStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyCycleStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_policy_cycle("code-195", "label-316");
        item.status = PolicyCycleStatus::Cancelled;
        assert!(
            item.move_to(PolicyCycleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
