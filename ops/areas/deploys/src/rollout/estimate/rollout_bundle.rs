use super::*;

use acme_kit::text;

/// Where a rollout bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutBundleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RolloutBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RolloutBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollout bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBundle {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub budget: f64,
    pub channel: String,
    pub retries: i64,
    pub status: RolloutBundleStatus,
}

impl RolloutBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutBundleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutBundleStatus) -> bool {
        match self.status {
            RolloutBundleStatus::Requested => {
                return next == RolloutBundleStatus::Revoked || next == RolloutBundleStatus::Expired;
            }
            RolloutBundleStatus::Granted => {
                match next {
                    RolloutBundleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutBundleStatus::Revoked => {
                match next {
                    RolloutBundleStatus::Expired => {
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

    pub fn move_to(&mut self, next: RolloutBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.port as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutBundle {
        RolloutBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            budget: self.budget,
            channel: self.channel.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rollout bundle in status requested.
pub fn new_rollout_bundle(code: &str, label: &str) -> RolloutBundle {
    RolloutBundle {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        budget: 0.0,
        channel: "".to_string(),
        retries: 0,
        status: RolloutBundleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rollout_bundle("code-597", "label-655");
        assert_eq!(item.status, RolloutBundleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rollout_bundle("code-597", "label-655");
        if item.move_to(RolloutBundleStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutBundleStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rollout_bundle("code-597", "label-655");
        item.status = RolloutBundleStatus::Expired;
        assert!(
            item.move_to(RolloutBundleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
