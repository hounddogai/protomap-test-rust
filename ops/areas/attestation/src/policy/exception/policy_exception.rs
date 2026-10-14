use super::*;

use acme_kit::text;

/// Where a policy exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyExceptionStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PolicyExceptionStatus {
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

impl std::fmt::Display for PolicyExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy exception as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyException {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub tags: Vec<String>,
    pub discount: f64,
    pub status: PolicyExceptionStatus,
}

impl PolicyException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyExceptionStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyExceptionStatus) -> bool {
        match self.status {
            PolicyExceptionStatus::Planned => {
                match next {
                    PolicyExceptionStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyExceptionStatus::Scheduled => {
                match next {
                    PolicyExceptionStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyExceptionStatus::InProgress => {
                match next {
                    PolicyExceptionStatus::Completed | PolicyExceptionStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyExceptionStatus::Completed => {
                match next {
                    PolicyExceptionStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PolicyExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.budget + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyException {
        PolicyException {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            tags: self.tags.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a policy exception in status planned.
pub fn new_policy_exception(code: &str, label: &str) -> PolicyException {
    PolicyException {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        tags: Vec::new(),
        discount: 0.0,
        status: PolicyExceptionStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_policy_exception("code-347", "label-336");
        assert_eq!(item.status, PolicyExceptionStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_policy_exception("code-347", "label-336");
        if item.move_to(PolicyExceptionStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyExceptionStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_policy_exception("code-347", "label-336");
        item.status = PolicyExceptionStatus::Cancelled;
        assert!(
            item.move_to(PolicyExceptionStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
