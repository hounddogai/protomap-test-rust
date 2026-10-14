use super::*;

use acme_kit::text;

/// Where a default limit policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultLimitPolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultLimitPolicyStatus {
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

impl std::fmt::Display for DefaultLimitPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default limit policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitPolicy {
    pub code: String,
    pub label: String,
    pub region: String,
    pub channel: String,
    pub pinned: bool,
    pub score: f64,
    pub status: DefaultLimitPolicyStatus,
}

impl DefaultLimitPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultLimitPolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultLimitPolicyStatus) -> bool {
        match self.status {
            DefaultLimitPolicyStatus::New => {
                match next {
                    DefaultLimitPolicyStatus::Queued | DefaultLimitPolicyStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultLimitPolicyStatus::Queued => {
                match next {
                    DefaultLimitPolicyStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultLimitPolicyStatus::Running => {
                return next == DefaultLimitPolicyStatus::Done;
            }
            DefaultLimitPolicyStatus::Done => {
                return next == DefaultLimitPolicyStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultLimitPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default limit policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default limit policy", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> DefaultLimitPolicy {
        DefaultLimitPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            channel: self.channel.clone(),
            pinned: self.pinned,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a default limit policy in status new.
pub fn new_default_limit_policy(code: &str, label: &str) -> DefaultLimitPolicy {
    DefaultLimitPolicy {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        channel: "".to_string(),
        pinned: false,
        score: 0.0,
        status: DefaultLimitPolicyStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_default_limit_policy("code-565", "label-11");
        assert_eq!(item.status, DefaultLimitPolicyStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_default_limit_policy("code-565", "label-11");
        if item.move_to(DefaultLimitPolicyStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultLimitPolicyStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_default_limit_policy("code-565", "label-11");
        item.status = DefaultLimitPolicyStatus::Failed;
        assert!(
            item.move_to(DefaultLimitPolicyStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
