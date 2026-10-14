use super::*;

use acme_kit::text;

/// Where a rollout policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutPolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RolloutPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RolloutPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout policy as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPolicy {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub currency: String,
    pub margin: f64,
    pub duration_minutes: i64,
    pub retries: i64,
    pub cost: f64,
    pub ceiling: f64,
    pub status: RolloutPolicyStatus,
}

impl RolloutPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutPolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutPolicyStatus) -> bool {
        match self.status {
            RolloutPolicyStatus::Requested => {
                return next == RolloutPolicyStatus::Expired;
            }
            RolloutPolicyStatus::Granted => {
                match next {
                    RolloutPolicyStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutPolicyStatus::Revoked => {
                return next == RolloutPolicyStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rollout policy", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.floor_value * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutPolicy {
        RolloutPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            currency: self.currency.clone(),
            margin: self.margin,
            duration_minutes: self.duration_minutes,
            retries: self.retries,
            cost: self.cost,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rollout policy in status requested.
pub fn new_rollout_policy(code: &str, label: &str) -> RolloutPolicy {
    RolloutPolicy {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        currency: "".to_string(),
        margin: 0.0,
        duration_minutes: 0,
        retries: 0,
        cost: 0.0,
        ceiling: 0.0,
        status: RolloutPolicyStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rollout_policy("code-134", "label-936");
        assert_eq!(item.status, RolloutPolicyStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_rollout_policy("code-134", "label-936");
        if item.move_to(RolloutPolicyStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutPolicyStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rollout_policy("code-134", "label-936");
        item.status = RolloutPolicyStatus::Expired;
        assert!(
            item.move_to(RolloutPolicyStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
