use super::*;

use acme_kit::text;

/// Where a strict rule preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRulePreferenceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl StrictRulePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for StrictRulePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict rule preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRulePreference {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub cost: f64,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub window_days: i64,
    pub notes: String,
    pub maximum: f64,
    pub status: StrictRulePreferenceStatus,
}

impl StrictRulePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRulePreferenceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRulePreferenceStatus) -> bool {
        match self.status {
            StrictRulePreferenceStatus::Proposed => {
                return next == StrictRulePreferenceStatus::Rejected || next == StrictRulePreferenceStatus::Withdrawn;
            }
            StrictRulePreferenceStatus::Approved => {
                return next == StrictRulePreferenceStatus::Rejected;
            }
            StrictRulePreferenceStatus::Rejected => {
                match next {
                    StrictRulePreferenceStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: StrictRulePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict rule preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "strict rule preference", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.cost - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictRulePreference {
        StrictRulePreference {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            cost: self.cost,
            tags: self.tags.clone(),
            pinned: self.pinned,
            window_days: self.window_days,
            notes: self.notes.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a strict rule preference in status proposed.
pub fn new_strict_rule_preference(code: &str, label: &str) -> StrictRulePreference {
    StrictRulePreference {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        cost: 0.0,
        tags: Vec::new(),
        pinned: false,
        window_days: 0,
        notes: "".to_string(),
        maximum: 0.0,
        status: StrictRulePreferenceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_strict_rule_preference("code-41", "label-806");
        assert_eq!(item.status, StrictRulePreferenceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_strict_rule_preference("code-41", "label-806");
        if item.move_to(StrictRulePreferenceStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRulePreferenceStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_strict_rule_preference("code-41", "label-806");
        item.status = StrictRulePreferenceStatus::Withdrawn;
        assert!(
            item.move_to(StrictRulePreferenceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
