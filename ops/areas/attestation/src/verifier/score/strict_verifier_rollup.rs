use super::*;

use acme_kit::text;

/// Where a strict verifier rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictVerifierRollupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictVerifierRollupStatus {
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

impl std::fmt::Display for StrictVerifierRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict verifier rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictVerifierRollup {
    pub code: String,
    pub name: String,
    pub country: String,
    pub expires_at: i64,
    pub budget: f64,
    pub created_at: i64,
    pub amount: f64,
    pub status: StrictVerifierRollupStatus,
}

impl StrictVerifierRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictVerifierRollupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictVerifierRollupStatus) -> bool {
        match self.status {
            StrictVerifierRollupStatus::Idle => {
                return next == StrictVerifierRollupStatus::Warming || next == StrictVerifierRollupStatus::Stopped;
            }
            StrictVerifierRollupStatus::Warming => {
                match next {
                    StrictVerifierRollupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictVerifierRollupStatus::Ready => {
                return next == StrictVerifierRollupStatus::Stopped;
            }
            StrictVerifierRollupStatus::Draining => {
                match next {
                    StrictVerifierRollupStatus::Stopped => {
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

    pub fn move_to(&mut self, next: StrictVerifierRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict verifier rollup {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.created_at as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictVerifierRollup {
        StrictVerifierRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            expires_at: self.expires_at,
            budget: self.budget,
            created_at: self.created_at,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict verifier rollup in status idle.
pub fn new_strict_verifier_rollup(code: &str, name: &str) -> StrictVerifierRollup {
    StrictVerifierRollup {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        expires_at: 0,
        budget: 0.0,
        created_at: 0,
        amount: 0.0,
        status: StrictVerifierRollupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_verifier_rollup("code-967", "name-356");
        assert_eq!(item.status, StrictVerifierRollupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_strict_verifier_rollup("code-967", "name-356");
        if item.move_to(StrictVerifierRollupStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictVerifierRollupStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_verifier_rollup("code-967", "name-356");
        item.status = StrictVerifierRollupStatus::Stopped;
        assert!(
            item.move_to(StrictVerifierRollupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
