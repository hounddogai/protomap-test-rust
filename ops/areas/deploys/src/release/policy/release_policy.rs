use super::*;

use acme_kit::text;

/// Where a release policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleasePolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReleasePolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ReleasePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePolicy {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub source: String,
    pub channel: String,
    pub pinned: bool,
    pub key_id: String,
    pub status: ReleasePolicyStatus,
}

impl ReleasePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleasePolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleasePolicyStatus) -> bool {
        match self.status {
            ReleasePolicyStatus::Idle => {
                return next == ReleasePolicyStatus::Stopped;
            }
            ReleasePolicyStatus::Warming => {
                match next {
                    ReleasePolicyStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleasePolicyStatus::Ready => {
                return next == ReleasePolicyStatus::Draining;
            }
            ReleasePolicyStatus::Draining => {
                return next == ReleasePolicyStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleasePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> ReleasePolicy {
        ReleasePolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            source: self.source.clone(),
            channel: self.channel.clone(),
            pinned: self.pinned,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a release policy in status idle.
pub fn new_release_policy(code: &str, label: &str) -> ReleasePolicy {
    ReleasePolicy {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        source: "".to_string(),
        channel: "".to_string(),
        pinned: false,
        key_id: "".to_string(),
        status: ReleasePolicyStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_release_policy("code-189", "label-185");
        assert_eq!(item.status, ReleasePolicyStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_release_policy("code-189", "label-185");
        if item.move_to(ReleasePolicyStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleasePolicyStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_release_policy("code-189", "label-185");
        item.status = ReleasePolicyStatus::Stopped;
        assert!(
            item.move_to(ReleasePolicyStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
