use super::*;

use acme_kit::text;

/// Where a canary policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryPolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CanaryPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for CanaryPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPolicy {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub pinned: bool,
    pub locale: String,
    pub status: CanaryPolicyStatus,
}

impl CanaryPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryPolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryPolicyStatus) -> bool {
        match self.status {
            CanaryPolicyStatus::New => {
                return next == CanaryPolicyStatus::Failed;
            }
            CanaryPolicyStatus::Queued => {
                match next {
                    CanaryPolicyStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryPolicyStatus::Running => {
                return next == CanaryPolicyStatus::Failed;
            }
            CanaryPolicyStatus::Done => {
                return next == CanaryPolicyStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> CanaryPolicy {
        CanaryPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            pinned: self.pinned,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a canary policy in status new.
pub fn new_canary_policy(code: &str, label: &str) -> CanaryPolicy {
    CanaryPolicy {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        pinned: false,
        locale: "".to_string(),
        status: CanaryPolicyStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_canary_policy("code-372", "label-672");
        assert_eq!(item.status, CanaryPolicyStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_canary_policy("code-372", "label-672");
        if item.move_to(CanaryPolicyStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryPolicyStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_canary_policy("code-372", "label-672");
        item.status = CanaryPolicyStatus::Failed;
        assert!(item.move_to(CanaryPolicyStatus::New).is_err(), "expected move to fail");
    }
}
