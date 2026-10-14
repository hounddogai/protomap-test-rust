use super::*;

use acme_kit::text;

/// Where a rollout override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutOverrideStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RolloutOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RolloutOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollout override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutOverride {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub phone: String,
    pub score: f64,
    pub ceiling: f64,
    pub status: RolloutOverrideStatus,
}

impl RolloutOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutOverrideStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutOverrideStatus) -> bool {
        match self.status {
            RolloutOverrideStatus::Draft => {
                return next == RolloutOverrideStatus::Closed;
            }
            RolloutOverrideStatus::Pending => {
                return next == RolloutOverrideStatus::Active || next == RolloutOverrideStatus::Suspended;
            }
            RolloutOverrideStatus::Active => {
                match next {
                    RolloutOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutOverrideStatus::Suspended => {
                return next == RolloutOverrideStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.expires_at as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutOverride {
        RolloutOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            phone: self.phone.clone(),
            score: self.score,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rollout override in status draft.
pub fn new_rollout_override(code: &str, label: &str) -> RolloutOverride {
    RolloutOverride {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        phone: "".to_string(),
        score: 0.0,
        ceiling: 0.0,
        status: RolloutOverrideStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rollout_override("code-772", "label-45");
        assert_eq!(item.status, RolloutOverrideStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_rollout_override("code-772", "label-45");
        if item.move_to(RolloutOverrideStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutOverrideStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rollout_override("code-772", "label-45");
        item.status = RolloutOverrideStatus::Closed;
        assert!(
            item.move_to(RolloutOverrideStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
