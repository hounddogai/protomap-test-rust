use super::*;

use acme_kit::text;

/// Where a usage policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsagePolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl UsagePolicyStatus {
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

impl std::fmt::Display for UsagePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage policy as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsagePolicy {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub tenant_id: String,
    pub updated_at: i64,
    pub status: UsagePolicyStatus,
}

impl UsagePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsagePolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsagePolicyStatus) -> bool {
        match self.status {
            UsagePolicyStatus::Planned => {
                match next {
                    UsagePolicyStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsagePolicyStatus::Scheduled => {
                match next {
                    UsagePolicyStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsagePolicyStatus::InProgress => {
                match next {
                    UsagePolicyStatus::Completed | UsagePolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsagePolicyStatus::Completed => {
                match next {
                    UsagePolicyStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: UsagePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage policy {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.retries as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsagePolicy {
        UsagePolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            tenant_id: self.tenant_id.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a usage policy in status planned.
pub fn new_usage_policy(code: &str, label: &str) -> UsagePolicy {
    UsagePolicy {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        tenant_id: "".to_string(),
        updated_at: 0,
        status: UsagePolicyStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_usage_policy("code-474", "label-747");
        assert_eq!(item.status, UsagePolicyStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_usage_policy("code-474", "label-747");
        if item.move_to(UsagePolicyStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsagePolicyStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_usage_policy("code-474", "label-747");
        item.status = UsagePolicyStatus::Cancelled;
        assert!(
            item.move_to(UsagePolicyStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
