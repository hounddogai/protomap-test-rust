use super::*;

use acme_kit::text;

/// Where a rolling verifier claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingVerifierClaimStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingVerifierClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RollingVerifierClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling verifier claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierClaim {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub amount: f64,
    pub limit: i64,
    pub maximum: f64,
    pub unit: String,
    pub status: RollingVerifierClaimStatus,
}

impl RollingVerifierClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingVerifierClaimStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingVerifierClaimStatus) -> bool {
        match self.status {
            RollingVerifierClaimStatus::New => {
                match next {
                    RollingVerifierClaimStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingVerifierClaimStatus::Queued => {
                return next == RollingVerifierClaimStatus::Running || next == RollingVerifierClaimStatus::Failed;
            }
            RollingVerifierClaimStatus::Running => {
                match next {
                    RollingVerifierClaimStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingVerifierClaimStatus::Done => {
                match next {
                    RollingVerifierClaimStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingVerifierClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling verifier claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.amount + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingVerifierClaim {
        RollingVerifierClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            amount: self.amount,
            limit: self.limit,
            maximum: self.maximum,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rolling verifier claim in status new.
pub fn new_rolling_verifier_claim(code: &str, name: &str) -> RollingVerifierClaim {
    RollingVerifierClaim {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        amount: 0.0,
        limit: 0,
        maximum: 0.0,
        unit: "".to_string(),
        status: RollingVerifierClaimStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_verifier_claim("code-208", "name-856");
        assert_eq!(item.status, RollingVerifierClaimStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_rolling_verifier_claim("code-208", "name-856");
        if item.move_to(RollingVerifierClaimStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingVerifierClaimStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_verifier_claim("code-208", "name-856");
        item.status = RollingVerifierClaimStatus::Failed;
        assert!(
            item.move_to(RollingVerifierClaimStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
