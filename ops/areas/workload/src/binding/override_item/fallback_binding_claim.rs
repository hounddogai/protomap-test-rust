use super::*;

use acme_kit::text;

/// Where a fallback binding claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackBindingClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FallbackBindingClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FallbackBindingClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback binding claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingClaim {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub threshold: f64,
    pub active: bool,
    pub status: FallbackBindingClaimStatus,
}

impl FallbackBindingClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackBindingClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackBindingClaimStatus) -> bool {
        match self.status {
            FallbackBindingClaimStatus::Proposed => {
                return next == FallbackBindingClaimStatus::Rejected || next == FallbackBindingClaimStatus::Withdrawn;
            }
            FallbackBindingClaimStatus::Approved => {
                match next {
                    FallbackBindingClaimStatus::Rejected | FallbackBindingClaimStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackBindingClaimStatus::Rejected => {
                match next {
                    FallbackBindingClaimStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FallbackBindingClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback binding claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> FallbackBindingClaim {
        FallbackBindingClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            threshold: self.threshold,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback binding claim in status proposed.
pub fn new_fallback_binding_claim(code: &str, label: &str) -> FallbackBindingClaim {
    FallbackBindingClaim {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        threshold: 0.0,
        active: false,
        status: FallbackBindingClaimStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_fallback_binding_claim("code-666", "label-538");
        assert_eq!(item.status, FallbackBindingClaimStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_fallback_binding_claim("code-666", "label-538");
        if item.move_to(FallbackBindingClaimStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackBindingClaimStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_fallback_binding_claim("code-666", "label-538");
        item.status = FallbackBindingClaimStatus::Withdrawn;
        assert!(
            item.move_to(FallbackBindingClaimStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
