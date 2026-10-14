use super::*;

use acme_kit::text;

/// Where a ttl policy score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TtlPolicyScoreStatus {
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

impl std::fmt::Display for TtlPolicyScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a ttl policy score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyScore {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub ttl_seconds: i64,
    pub expires_at: i64,
    pub status: TtlPolicyScoreStatus,
}

impl TtlPolicyScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyScoreStatus) -> bool {
        match self.status {
            TtlPolicyScoreStatus::Idle => {
                match next {
                    TtlPolicyScoreStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyScoreStatus::Warming => {
                match next {
                    TtlPolicyScoreStatus::Ready | TtlPolicyScoreStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyScoreStatus::Ready => {
                return next == TtlPolicyScoreStatus::Draining || next == TtlPolicyScoreStatus::Stopped;
            }
            TtlPolicyScoreStatus::Draining => {
                match next {
                    TtlPolicyScoreStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TtlPolicyScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy score {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "ttl policy score", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.expires_at as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyScore {
        TtlPolicyScore {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            ttl_seconds: self.ttl_seconds,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a ttl policy score in status idle.
pub fn new_ttl_policy_score(code: &str, title: &str) -> TtlPolicyScore {
    TtlPolicyScore {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        ttl_seconds: 0,
        expires_at: 0,
        status: TtlPolicyScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_ttl_policy_score("code-354", "title-498");
        assert_eq!(item.status, TtlPolicyScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_ttl_policy_score("code-354", "title-498");
        if item.move_to(TtlPolicyScoreStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyScoreStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_ttl_policy_score("code-354", "title-498");
        item.status = TtlPolicyScoreStatus::Stopped;
        assert!(
            item.move_to(TtlPolicyScoreStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
