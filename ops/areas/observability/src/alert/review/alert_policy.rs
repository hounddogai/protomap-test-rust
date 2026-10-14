use super::*;

use acme_kit::text;

/// Where a alert policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertPolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AlertPolicyStatus {
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

impl std::fmt::Display for AlertPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert policy as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertPolicy {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub ends_at: i64,
    pub score: f64,
    pub quantity: i64,
    pub status: AlertPolicyStatus,
}

impl AlertPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertPolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertPolicyStatus) -> bool {
        match self.status {
            AlertPolicyStatus::Idle => {
                match next {
                    AlertPolicyStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertPolicyStatus::Warming => {
                return next == AlertPolicyStatus::Ready;
            }
            AlertPolicyStatus::Ready => {
                match next {
                    AlertPolicyStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertPolicyStatus::Draining => {
                match next {
                    AlertPolicyStatus::Stopped => {
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

    pub fn move_to(&mut self, next: AlertPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "alert policy", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.quantity as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertPolicy {
        AlertPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            ends_at: self.ends_at,
            score: self.score,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a alert policy in status idle.
pub fn new_alert_policy(code: &str, name: &str) -> AlertPolicy {
    AlertPolicy {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        ends_at: 0,
        score: 0.0,
        quantity: 0,
        status: AlertPolicyStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_alert_policy("code-981", "name-452");
        assert_eq!(item.status, AlertPolicyStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_alert_policy("code-981", "name-452");
        if item.move_to(AlertPolicyStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertPolicyStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_alert_policy("code-981", "name-452");
        item.status = AlertPolicyStatus::Stopped;
        assert!(item.move_to(AlertPolicyStatus::Idle).is_err(), "expected move to fail");
    }
}
