use super::*;

use acme_kit::text;

/// Where a egress review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EgressReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EgressReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressReview {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub quota_units: i64,
    pub issuer: String,
    pub status: EgressReviewStatus,
}

impl EgressReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressReviewStatus) -> bool {
        match self.status {
            EgressReviewStatus::Created => {
                return next == EgressReviewStatus::Validated;
            }
            EgressReviewStatus::Validated => {
                return next == EgressReviewStatus::Published || next == EgressReviewStatus::Retired;
            }
            EgressReviewStatus::Published => {
                return next == EgressReviewStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "egress review", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.grace_days as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressReview {
        EgressReview {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            quota_units: self.quota_units,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a egress review in status created.
pub fn new_egress_review(code: &str, label: &str) -> EgressReview {
    EgressReview {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        quota_units: 0,
        issuer: "".to_string(),
        status: EgressReviewStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_egress_review("code-409", "label-796");
        assert_eq!(item.status, EgressReviewStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_egress_review("code-409", "label-796");
        if item.move_to(EgressReviewStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressReviewStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_egress_review("code-409", "label-796");
        item.status = EgressReviewStatus::Retired;
        assert!(
            item.move_to(EgressReviewStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
