use super::*;

use acme_kit::text;

/// Where a weighted policy draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedPolicyDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedPolicyDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedPolicyDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted policy draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyDraft {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub scope: String,
    pub priority: i64,
    pub margin: f64,
    pub contact_email: String,
    pub status: WeightedPolicyDraftStatus,
}

impl WeightedPolicyDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedPolicyDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedPolicyDraftStatus) -> bool {
        match self.status {
            WeightedPolicyDraftStatus::Created => {
                match next {
                    WeightedPolicyDraftStatus::Validated | WeightedPolicyDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPolicyDraftStatus::Validated => {
                match next {
                    WeightedPolicyDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPolicyDraftStatus::Published => {
                match next {
                    WeightedPolicyDraftStatus::Retired => {
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

    pub fn move_to(&mut self, next: WeightedPolicyDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted policy draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ratio - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedPolicyDraft {
        WeightedPolicyDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            scope: self.scope.clone(),
            priority: self.priority,
            margin: self.margin,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a weighted policy draft in status created.
pub fn new_weighted_policy_draft(code: &str, label: &str) -> WeightedPolicyDraft {
    WeightedPolicyDraft {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        scope: "".to_string(),
        priority: 0,
        margin: 0.0,
        contact_email: "".to_string(),
        status: WeightedPolicyDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_policy_draft("code-223", "label-686");
        assert_eq!(item.status, WeightedPolicyDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_weighted_policy_draft("code-223", "label-686");
        if item.move_to(WeightedPolicyDraftStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedPolicyDraftStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_policy_draft("code-223", "label-686");
        item.status = WeightedPolicyDraftStatus::Retired;
        assert!(
            item.move_to(WeightedPolicyDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
