use super::*;

use acme_kit::text;

/// Where a primary evidence claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEvidenceClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryEvidenceClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryEvidenceClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary evidence claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceClaim {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub quantity: i64,
    pub amount: f64,
    pub updated_at: i64,
    pub status: PrimaryEvidenceClaimStatus,
}

impl PrimaryEvidenceClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEvidenceClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEvidenceClaimStatus) -> bool {
        match self.status {
            PrimaryEvidenceClaimStatus::Proposed => {
                match next {
                    PrimaryEvidenceClaimStatus::Approved | PrimaryEvidenceClaimStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEvidenceClaimStatus::Approved => {
                match next {
                    PrimaryEvidenceClaimStatus::Rejected | PrimaryEvidenceClaimStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEvidenceClaimStatus::Rejected => {
                return next == PrimaryEvidenceClaimStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryEvidenceClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary evidence claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary evidence claim", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.updated_at as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryEvidenceClaim {
        PrimaryEvidenceClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            quantity: self.quantity,
            amount: self.amount,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary evidence claim in status proposed.
pub fn new_primary_evidence_claim(code: &str, label: &str) -> PrimaryEvidenceClaim {
    PrimaryEvidenceClaim {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        quantity: 0,
        amount: 0.0,
        updated_at: 0,
        status: PrimaryEvidenceClaimStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_evidence_claim("code-382", "label-69");
        assert_eq!(item.status, PrimaryEvidenceClaimStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_primary_evidence_claim("code-382", "label-69");
        if item.move_to(PrimaryEvidenceClaimStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryEvidenceClaimStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_evidence_claim("code-382", "label-69");
        item.status = PrimaryEvidenceClaimStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryEvidenceClaimStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
