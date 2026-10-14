use super::*;

use acme_kit::text;

/// Where a peering agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PeeringAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PeeringAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering agreement as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAgreement {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub priority: i64,
    pub budget: f64,
    pub attempts: i64,
    pub status: PeeringAgreementStatus,
}

impl PeeringAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringAgreementStatus) -> bool {
        match self.status {
            PeeringAgreementStatus::Open => {
                match next {
                    PeeringAgreementStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringAgreementStatus::InReview => {
                return next == PeeringAgreementStatus::Reopened || next == PeeringAgreementStatus::Archived;
            }
            PeeringAgreementStatus::Resolved => {
                return next == PeeringAgreementStatus::Reopened || next == PeeringAgreementStatus::Archived;
            }
            PeeringAgreementStatus::Reopened => {
                match next {
                    PeeringAgreementStatus::Archived => {
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

    pub fn move_to(&mut self, next: PeeringAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "peering agreement", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.attempts as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringAgreement {
        PeeringAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            priority: self.priority,
            budget: self.budget,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a peering agreement in status open.
pub fn new_peering_agreement(code: &str, name: &str) -> PeeringAgreement {
    PeeringAgreement {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        priority: 0,
        budget: 0.0,
        attempts: 0,
        status: PeeringAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_peering_agreement("code-793", "name-803");
        assert_eq!(item.status, PeeringAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_peering_agreement("code-793", "name-803");
        if item.move_to(PeeringAgreementStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringAgreementStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_peering_agreement("code-793", "name-803");
        item.status = PeeringAgreementStatus::Archived;
        assert!(
            item.move_to(PeeringAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
