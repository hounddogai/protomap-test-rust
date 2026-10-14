use super::*;

use acme_kit::text;

/// Where a fallback verifier estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackVerifierEstimateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FallbackVerifierEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FallbackVerifierEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback verifier estimate as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierEstimate {
    pub code: String,
    pub name: String,
    pub description: String,
    pub batch_size: i64,
    pub threshold: f64,
    pub status: FallbackVerifierEstimateStatus,
}

impl FallbackVerifierEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackVerifierEstimateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackVerifierEstimateStatus) -> bool {
        match self.status {
            FallbackVerifierEstimateStatus::Proposed => {
                return next == FallbackVerifierEstimateStatus::Approved;
            }
            FallbackVerifierEstimateStatus::Approved => {
                return next == FallbackVerifierEstimateStatus::Rejected;
            }
            FallbackVerifierEstimateStatus::Rejected => {
                match next {
                    FallbackVerifierEstimateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FallbackVerifierEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback verifier estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.batch_size as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackVerifierEstimate {
        FallbackVerifierEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            batch_size: self.batch_size,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback verifier estimate in status proposed.
pub fn new_fallback_verifier_estimate(code: &str, name: &str) -> FallbackVerifierEstimate {
    FallbackVerifierEstimate {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        batch_size: 0,
        threshold: 0.0,
        status: FallbackVerifierEstimateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_fallback_verifier_estimate("code-552", "name-30");
        assert_eq!(item.status, FallbackVerifierEstimateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_fallback_verifier_estimate("code-552", "name-30");
        if item.move_to(FallbackVerifierEstimateStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackVerifierEstimateStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_fallback_verifier_estimate("code-552", "name-30");
        item.status = FallbackVerifierEstimateStatus::Withdrawn;
        assert!(
            item.move_to(FallbackVerifierEstimateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
