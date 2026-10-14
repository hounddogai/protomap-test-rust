use super::*;

use acme_kit::text;

/// Where a delegation preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationPreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DelegationPreferenceStatus {
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

impl std::fmt::Display for DelegationPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a delegation preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPreference {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub scope: String,
    pub threshold: f64,
    pub currency: String,
    pub status: DelegationPreferenceStatus,
}

impl DelegationPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationPreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationPreferenceStatus) -> bool {
        match self.status {
            DelegationPreferenceStatus::Open => {
                return next == DelegationPreferenceStatus::Resolved;
            }
            DelegationPreferenceStatus::InReview => {
                return next == DelegationPreferenceStatus::Reopened || next == DelegationPreferenceStatus::Archived;
            }
            DelegationPreferenceStatus::Resolved => {
                match next {
                    DelegationPreferenceStatus::Reopened | DelegationPreferenceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationPreferenceStatus::Reopened => {
                return next == DelegationPreferenceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.discount * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationPreference {
        DelegationPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            scope: self.scope.clone(),
            threshold: self.threshold,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a delegation preference in status open.
pub fn new_delegation_preference(code: &str, title: &str) -> DelegationPreference {
    DelegationPreference {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        scope: "".to_string(),
        threshold: 0.0,
        currency: "".to_string(),
        status: DelegationPreferenceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_delegation_preference("code-806", "title-986");
        assert_eq!(item.status, DelegationPreferenceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_delegation_preference("code-806", "title-986");
        if item.move_to(DelegationPreferenceStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationPreferenceStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_delegation_preference("code-806", "title-986");
        item.status = DelegationPreferenceStatus::Archived;
        assert!(
            item.move_to(DelegationPreferenceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
