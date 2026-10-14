use super::*;

use acme_kit::text;

/// Where a rollout request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutRequestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RolloutRequestStatus {
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

impl std::fmt::Display for RolloutRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollout request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRequest {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub rate: f64,
    pub threshold: f64,
    pub capacity: i64,
    pub tenant_id: String,
    pub status: RolloutRequestStatus,
}

impl RolloutRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutRequestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutRequestStatus) -> bool {
        match self.status {
            RolloutRequestStatus::Idle => {
                match next {
                    RolloutRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutRequestStatus::Warming => {
                match next {
                    RolloutRequestStatus::Ready | RolloutRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutRequestStatus::Ready => {
                match next {
                    RolloutRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutRequestStatus::Draining => {
                return next == RolloutRequestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.capacity as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutRequest {
        RolloutRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            rate: self.rate,
            threshold: self.threshold,
            capacity: self.capacity,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rollout request in status idle.
pub fn new_rollout_request(code: &str, title: &str) -> RolloutRequest {
    RolloutRequest {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        rate: 0.0,
        threshold: 0.0,
        capacity: 0,
        tenant_id: "".to_string(),
        status: RolloutRequestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rollout_request("code-940", "title-291");
        assert_eq!(item.status, RolloutRequestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_rollout_request("code-940", "title-291");
        if item.move_to(RolloutRequestStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutRequestStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rollout_request("code-940", "title-291");
        item.status = RolloutRequestStatus::Stopped;
        assert!(
            item.move_to(RolloutRequestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
