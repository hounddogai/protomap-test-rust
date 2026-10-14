use super::*;

use acme_kit::text;

/// Where a usage checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageCheckpointStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl UsageCheckpointStatus {
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

impl std::fmt::Display for UsageCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageCheckpoint {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub contact_email: String,
    pub country: String,
    pub fingerprint: String,
    pub locale: String,
    pub status: UsageCheckpointStatus,
}

impl UsageCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageCheckpointStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageCheckpointStatus) -> bool {
        match self.status {
            UsageCheckpointStatus::Idle => {
                return next == UsageCheckpointStatus::Warming || next == UsageCheckpointStatus::Draining;
            }
            UsageCheckpointStatus::Warming => {
                return next == UsageCheckpointStatus::Ready || next == UsageCheckpointStatus::Draining;
            }
            UsageCheckpointStatus::Ready => {
                return next == UsageCheckpointStatus::Draining;
            }
            UsageCheckpointStatus::Draining => {
                match next {
                    UsageCheckpointStatus::Stopped => {
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

    pub fn move_to(&mut self, next: UsageCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> UsageCheckpoint {
        UsageCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            contact_email: self.contact_email.clone(),
            country: self.country.clone(),
            fingerprint: self.fingerprint.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a usage checkpoint in status idle.
pub fn new_usage_checkpoint(code: &str, name: &str) -> UsageCheckpoint {
    UsageCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        contact_email: "".to_string(),
        country: "".to_string(),
        fingerprint: "".to_string(),
        locale: "".to_string(),
        status: UsageCheckpointStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_usage_checkpoint("code-157", "name-832");
        assert_eq!(item.status, UsageCheckpointStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_usage_checkpoint("code-157", "name-832");
        if item.move_to(UsageCheckpointStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageCheckpointStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_usage_checkpoint("code-157", "name-832");
        item.status = UsageCheckpointStatus::Stopped;
        assert!(
            item.move_to(UsageCheckpointStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
