use super::*;

use acme_kit::text;

/// Where a binding review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BindingReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BindingReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One binding review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingReview {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub cost: f64,
    pub notes: String,
    pub amount: f64,
    pub ceiling: f64,
    pub cipher: String,
    pub scope: String,
    pub source: String,
    pub status: BindingReviewStatus,
}

impl BindingReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingReviewStatus) -> bool {
        match self.status {
            BindingReviewStatus::Proposed => {
                return next == BindingReviewStatus::Approved || next == BindingReviewStatus::Rejected;
            }
            BindingReviewStatus::Approved => {
                return next == BindingReviewStatus::Rejected;
            }
            BindingReviewStatus::Rejected => {
                return next == BindingReviewStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.cost - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingReview {
        BindingReview {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            cost: self.cost,
            notes: self.notes.clone(),
            amount: self.amount,
            ceiling: self.ceiling,
            cipher: self.cipher.clone(),
            scope: self.scope.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a binding review in status proposed.
pub fn new_binding_review(code: &str, label: &str) -> BindingReview {
    BindingReview {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        cost: 0.0,
        notes: "".to_string(),
        amount: 0.0,
        ceiling: 0.0,
        cipher: "".to_string(),
        scope: "".to_string(),
        source: "".to_string(),
        status: BindingReviewStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_binding_review("code-925", "label-597");
        assert_eq!(item.status, BindingReviewStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_binding_review("code-925", "label-597");
        if item.move_to(BindingReviewStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingReviewStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_binding_review("code-925", "label-597");
        item.status = BindingReviewStatus::Withdrawn;
        assert!(
            item.move_to(BindingReviewStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
