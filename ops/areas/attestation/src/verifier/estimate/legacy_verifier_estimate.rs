use super::*;

use acme_kit::text;

/// Where a legacy verifier estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyVerifierEstimateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyVerifierEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyVerifierEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy verifier estimate as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierEstimate {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub rotation_days: i64,
    pub starts_at: i64,
    pub window_days: i64,
    pub threshold: f64,
    pub quantity: i64,
    pub status: LegacyVerifierEstimateStatus,
}

impl LegacyVerifierEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyVerifierEstimateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyVerifierEstimateStatus) -> bool {
        match self.status {
            LegacyVerifierEstimateStatus::Proposed => {
                match next {
                    LegacyVerifierEstimateStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyVerifierEstimateStatus::Approved => {
                return next == LegacyVerifierEstimateStatus::Rejected
                    || next == LegacyVerifierEstimateStatus::Withdrawn;
            }
            LegacyVerifierEstimateStatus::Rejected => {
                return next == LegacyVerifierEstimateStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyVerifierEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy verifier estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.window_days as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyVerifierEstimate {
        LegacyVerifierEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            rotation_days: self.rotation_days,
            starts_at: self.starts_at,
            window_days: self.window_days,
            threshold: self.threshold,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy verifier estimate in status proposed.
pub fn new_legacy_verifier_estimate(code: &str, label: &str) -> LegacyVerifierEstimate {
    LegacyVerifierEstimate {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        rotation_days: 0,
        starts_at: 0,
        window_days: 0,
        threshold: 0.0,
        quantity: 0,
        status: LegacyVerifierEstimateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_legacy_verifier_estimate("code-97", "label-200");
        assert_eq!(item.status, LegacyVerifierEstimateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_legacy_verifier_estimate("code-97", "label-200");
        if item.move_to(LegacyVerifierEstimateStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyVerifierEstimateStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_legacy_verifier_estimate("code-97", "label-200");
        item.status = LegacyVerifierEstimateStatus::Withdrawn;
        assert!(
            item.move_to(LegacyVerifierEstimateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
