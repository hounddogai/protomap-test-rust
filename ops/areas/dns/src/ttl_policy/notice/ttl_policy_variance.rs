use super::*;

use acme_kit::text;

/// Where a ttl policy variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TtlPolicyVarianceStatus {
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

impl std::fmt::Display for TtlPolicyVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyVariance {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub pinned: bool,
    pub locale: String,
    pub ceiling: f64,
    pub status: TtlPolicyVarianceStatus,
}

impl TtlPolicyVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyVarianceStatus) -> bool {
        match self.status {
            TtlPolicyVarianceStatus::Planned => {
                match next {
                    TtlPolicyVarianceStatus::Scheduled | TtlPolicyVarianceStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyVarianceStatus::Scheduled => {
                return next == TtlPolicyVarianceStatus::Cancelled;
            }
            TtlPolicyVarianceStatus::InProgress => {
                return next == TtlPolicyVarianceStatus::Cancelled;
            }
            TtlPolicyVarianceStatus::Completed => {
                return next == TtlPolicyVarianceStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "ttl policy variance", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.attempts as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyVariance {
        TtlPolicyVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            pinned: self.pinned,
            locale: self.locale.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a ttl policy variance in status planned.
pub fn new_ttl_policy_variance(code: &str, label: &str) -> TtlPolicyVariance {
    TtlPolicyVariance {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        pinned: false,
        locale: "".to_string(),
        ceiling: 0.0,
        status: TtlPolicyVarianceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_ttl_policy_variance("code-852", "label-310");
        assert_eq!(item.status, TtlPolicyVarianceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_ttl_policy_variance("code-852", "label-310");
        if item.move_to(TtlPolicyVarianceStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyVarianceStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_ttl_policy_variance("code-852", "label-310");
        item.status = TtlPolicyVarianceStatus::Cancelled;
        assert!(
            item.move_to(TtlPolicyVarianceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
