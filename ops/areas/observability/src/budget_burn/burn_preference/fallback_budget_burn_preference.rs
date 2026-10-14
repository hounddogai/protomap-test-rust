use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback budget burn preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackBudgetBurnPreferenceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FallbackBudgetBurnPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FallbackBudgetBurnPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback budget burn preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnPreference {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub expires_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub summary: String,
    pub score: f64,
    pub rate: f64,
    pub floor_value: f64,
    pub status: FallbackBudgetBurnPreferenceStatus,
}

impl FallbackBudgetBurnPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackBudgetBurnPreferenceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackBudgetBurnPreferenceStatus) -> bool {
        match self.status {
            FallbackBudgetBurnPreferenceStatus::Proposed => {
                return next == FallbackBudgetBurnPreferenceStatus::Withdrawn;
            }
            FallbackBudgetBurnPreferenceStatus::Approved => {
                return next == FallbackBudgetBurnPreferenceStatus::Rejected
                    || next == FallbackBudgetBurnPreferenceStatus::Withdrawn;
            }
            FallbackBudgetBurnPreferenceStatus::Rejected => {
                match next {
                    FallbackBudgetBurnPreferenceStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FallbackBudgetBurnPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback budget burn preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.floor_value + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackBudgetBurnPreference {
        FallbackBudgetBurnPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            expires_at: self.expires_at,
            attributes: self.attributes.clone(),
            summary: self.summary.clone(),
            score: self.score,
            rate: self.rate,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a fallback budget burn preference in status proposed.
pub fn new_fallback_budget_burn_preference(code: &str, name: &str) -> FallbackBudgetBurnPreference {
    FallbackBudgetBurnPreference {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        expires_at: 0,
        attributes: BTreeMap::new(),
        summary: "".to_string(),
        score: 0.0,
        rate: 0.0,
        floor_value: 0.0,
        status: FallbackBudgetBurnPreferenceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_fallback_budget_burn_preference("code-529", "name-629");
        assert_eq!(item.status, FallbackBudgetBurnPreferenceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_fallback_budget_burn_preference("code-529", "name-629");
        if item.move_to(FallbackBudgetBurnPreferenceStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackBudgetBurnPreferenceStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_fallback_budget_burn_preference("code-529", "name-629");
        item.status = FallbackBudgetBurnPreferenceStatus::Withdrawn;
        assert!(
            item.move_to(FallbackBudgetBurnPreferenceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
