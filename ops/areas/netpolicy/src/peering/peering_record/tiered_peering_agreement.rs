use super::*;

use acme_kit::text;

/// Where a tiered peering agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPeeringAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredPeeringAgreementStatus {
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

impl std::fmt::Display for TieredPeeringAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered peering agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAgreement {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub threshold: f64,
    pub label: String,
    pub external_ref: String,
    pub status: TieredPeeringAgreementStatus,
}

impl TieredPeeringAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPeeringAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPeeringAgreementStatus) -> bool {
        match self.status {
            TieredPeeringAgreementStatus::Open => {
                match next {
                    TieredPeeringAgreementStatus::Resolved | TieredPeeringAgreementStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPeeringAgreementStatus::InReview => {
                return next == TieredPeeringAgreementStatus::Reopened;
            }
            TieredPeeringAgreementStatus::Resolved => {
                match next {
                    TieredPeeringAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPeeringAgreementStatus::Reopened => {
                match next {
                    TieredPeeringAgreementStatus::Archived => {
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

    pub fn move_to(&mut self, next: TieredPeeringAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered peering agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> TieredPeeringAgreement {
        TieredPeeringAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            threshold: self.threshold,
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered peering agreement in status open.
pub fn new_tiered_peering_agreement(code: &str, title: &str) -> TieredPeeringAgreement {
    TieredPeeringAgreement {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        threshold: 0.0,
        label: "".to_string(),
        external_ref: "".to_string(),
        status: TieredPeeringAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_peering_agreement("code-496", "title-873");
        assert_eq!(item.status, TieredPeeringAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_tiered_peering_agreement("code-496", "title-873");
        if item.move_to(TieredPeeringAgreementStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredPeeringAgreementStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_peering_agreement("code-496", "title-873");
        item.status = TieredPeeringAgreementStatus::Archived;
        assert!(
            item.move_to(TieredPeeringAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
