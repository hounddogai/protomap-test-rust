use super::*;

use acme_kit::text;

/// Where a overage claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl OverageClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for OverageClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage claim as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageClaim {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub summary: String,
    pub fingerprint: String,
    pub threshold: f64,
    pub unit: String,
    pub status: OverageClaimStatus,
}

impl OverageClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageClaimStatus) -> bool {
        match self.status {
            OverageClaimStatus::Proposed => {
                return next == OverageClaimStatus::Approved;
            }
            OverageClaimStatus::Approved => {
                return next == OverageClaimStatus::Withdrawn;
            }
            OverageClaimStatus::Rejected => {
                return next == OverageClaimStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> OverageClaim {
        OverageClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            summary: self.summary.clone(),
            fingerprint: self.fingerprint.clone(),
            threshold: self.threshold,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a overage claim in status proposed.
pub fn new_overage_claim(code: &str, title: &str) -> OverageClaim {
    OverageClaim {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        summary: "".to_string(),
        fingerprint: "".to_string(),
        threshold: 0.0,
        unit: "".to_string(),
        status: OverageClaimStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_overage_claim("code-804", "title-484");
        assert_eq!(item.status, OverageClaimStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_overage_claim("code-804", "title-484");
        if item.move_to(OverageClaimStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageClaimStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_overage_claim("code-804", "title-484");
        item.status = OverageClaimStatus::Withdrawn;
        assert!(
            item.move_to(OverageClaimStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
