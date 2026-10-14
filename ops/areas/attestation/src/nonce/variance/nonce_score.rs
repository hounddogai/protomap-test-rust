use super::*;

use acme_kit::text;

/// Where a nonce score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl NonceScoreStatus {
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

impl std::fmt::Display for NonceScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce score as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceScore {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub rotation_days: i64,
    pub pinned: bool,
    pub rate: f64,
    pub quota_units: i64,
    pub attempts: i64,
    pub limit: i64,
    pub grace_days: i64,
    pub status: NonceScoreStatus,
}

impl NonceScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceScoreStatus) -> bool {
        match self.status {
            NonceScoreStatus::Idle => {
                match next {
                    NonceScoreStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceScoreStatus::Warming => {
                match next {
                    NonceScoreStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceScoreStatus::Ready => {
                return next == NonceScoreStatus::Draining || next == NonceScoreStatus::Stopped;
            }
            NonceScoreStatus::Draining => {
                return next == NonceScoreStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "nonce score", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.grace_days as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceScore {
        NonceScore {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            rotation_days: self.rotation_days,
            pinned: self.pinned,
            rate: self.rate,
            quota_units: self.quota_units,
            attempts: self.attempts,
            limit: self.limit,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a nonce score in status idle.
pub fn new_nonce_score(code: &str, name: &str) -> NonceScore {
    NonceScore {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        rotation_days: 0,
        pinned: false,
        rate: 0.0,
        quota_units: 0,
        attempts: 0,
        limit: 0,
        grace_days: 0,
        status: NonceScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_nonce_score("code-753", "name-390");
        assert_eq!(item.status, NonceScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_nonce_score("code-753", "name-390");
        if item.move_to(NonceScoreStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceScoreStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_nonce_score("code-753", "name-390");
        item.status = NonceScoreStatus::Stopped;
        assert!(item.move_to(NonceScoreStatus::Idle).is_err(), "expected move to fail");
    }
}
