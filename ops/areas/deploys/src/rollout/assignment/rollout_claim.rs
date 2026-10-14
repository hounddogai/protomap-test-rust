use super::*;

use acme_kit::text;

/// Where a rollout claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RolloutClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RolloutClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollout claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutClaim {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub minimum: f64,
    pub approved: bool,
    pub contact_email: String,
    pub ratio: f64,
    pub notes: String,
    pub margin: f64,
    pub quantity: i64,
    pub scope: String,
    pub status: RolloutClaimStatus,
}

impl RolloutClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutClaimStatus) -> bool {
        match self.status {
            RolloutClaimStatus::Proposed => {
                return next == RolloutClaimStatus::Rejected;
            }
            RolloutClaimStatus::Approved => {
                return next == RolloutClaimStatus::Rejected || next == RolloutClaimStatus::Withdrawn;
            }
            RolloutClaimStatus::Rejected => {
                return next == RolloutClaimStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollout claim", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ratio - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutClaim {
        RolloutClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            minimum: self.minimum,
            approved: self.approved,
            contact_email: self.contact_email.clone(),
            ratio: self.ratio,
            notes: self.notes.clone(),
            margin: self.margin,
            quantity: self.quantity,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a rollout claim in status proposed.
pub fn new_rollout_claim(code: &str, title: &str) -> RolloutClaim {
    RolloutClaim {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        minimum: 0.0,
        approved: false,
        contact_email: "".to_string(),
        ratio: 0.0,
        notes: "".to_string(),
        margin: 0.0,
        quantity: 0,
        scope: "".to_string(),
        status: RolloutClaimStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rollout_claim("code-504", "title-603");
        assert_eq!(item.status, RolloutClaimStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_rollout_claim("code-504", "title-603");
        if item.move_to(RolloutClaimStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutClaimStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rollout_claim("code-504", "title-603");
        item.status = RolloutClaimStatus::Withdrawn;
        assert!(
            item.move_to(RolloutClaimStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
