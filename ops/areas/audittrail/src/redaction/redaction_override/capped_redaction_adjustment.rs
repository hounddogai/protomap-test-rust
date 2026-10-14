use super::*;

use acme_kit::text;

/// Where a capped redaction adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRedactionAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedRedactionAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedRedactionAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped redaction adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionAdjustment {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub description: String,
    pub category: String,
    pub amount: f64,
    pub status: CappedRedactionAdjustmentStatus,
}

impl CappedRedactionAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRedactionAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRedactionAdjustmentStatus) -> bool {
        match self.status {
            CappedRedactionAdjustmentStatus::Proposed => {
                return next == CappedRedactionAdjustmentStatus::Approved;
            }
            CappedRedactionAdjustmentStatus::Approved => {
                match next {
                    CappedRedactionAdjustmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRedactionAdjustmentStatus::Rejected => {
                return next == CappedRedactionAdjustmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedRedactionAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped redaction adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> CappedRedactionAdjustment {
        CappedRedactionAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            description: self.description.clone(),
            category: self.category.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a capped redaction adjustment in status proposed.
pub fn new_capped_redaction_adjustment(code: &str, title: &str) -> CappedRedactionAdjustment {
    CappedRedactionAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        description: "".to_string(),
        category: "".to_string(),
        amount: 0.0,
        status: CappedRedactionAdjustmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_redaction_adjustment("code-848", "title-49");
        assert_eq!(item.status, CappedRedactionAdjustmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_capped_redaction_adjustment("code-848", "title-49");
        if item.move_to(CappedRedactionAdjustmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedRedactionAdjustmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_redaction_adjustment("code-848", "title-49");
        item.status = CappedRedactionAdjustmentStatus::Withdrawn;
        assert!(
            item.move_to(CappedRedactionAdjustmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
