use super::*;

use acme_kit::text;

/// Where a retention policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionPolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RetentionPolicyStatus {
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

impl std::fmt::Display for RetentionPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention policy as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPolicy {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub weight: f64,
    pub created_at: i64,
    pub attempts: i64,
    pub ceiling: f64,
    pub status: RetentionPolicyStatus,
}

impl RetentionPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionPolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionPolicyStatus) -> bool {
        match self.status {
            RetentionPolicyStatus::Idle => {
                return next == RetentionPolicyStatus::Stopped;
            }
            RetentionPolicyStatus::Warming => {
                match next {
                    RetentionPolicyStatus::Draining | RetentionPolicyStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionPolicyStatus::Ready => {
                return next == RetentionPolicyStatus::Stopped;
            }
            RetentionPolicyStatus::Draining => {
                match next {
                    RetentionPolicyStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RetentionPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.attempts as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionPolicy {
        RetentionPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            weight: self.weight,
            created_at: self.created_at,
            attempts: self.attempts,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a retention policy in status idle.
pub fn new_retention_policy(code: &str, label: &str) -> RetentionPolicy {
    RetentionPolicy {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        weight: 0.0,
        created_at: 0,
        attempts: 0,
        ceiling: 0.0,
        status: RetentionPolicyStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_retention_policy("code-458", "label-716");
        assert_eq!(item.status, RetentionPolicyStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_retention_policy("code-458", "label-716");
        if item.move_to(RetentionPolicyStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionPolicyStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_retention_policy("code-458", "label-716");
        item.status = RetentionPolicyStatus::Stopped;
        assert!(
            item.move_to(RetentionPolicyStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
