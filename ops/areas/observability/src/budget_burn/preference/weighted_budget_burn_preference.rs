use super::*;

use acme_kit::text;

/// Where a weighted budget burn preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedBudgetBurnPreferenceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedBudgetBurnPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for WeightedBudgetBurnPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted budget burn preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnPreference {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub tenant_id: String,
    pub host: String,
    pub scope: String,
    pub ratio: f64,
    pub rate: f64,
    pub budget: f64,
    pub floor_value: f64,
    pub status: WeightedBudgetBurnPreferenceStatus,
}

impl WeightedBudgetBurnPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedBudgetBurnPreferenceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedBudgetBurnPreferenceStatus) -> bool {
        match self.status {
            WeightedBudgetBurnPreferenceStatus::New => {
                match next {
                    WeightedBudgetBurnPreferenceStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedBudgetBurnPreferenceStatus::Queued => {
                return next == WeightedBudgetBurnPreferenceStatus::Running
                    || next == WeightedBudgetBurnPreferenceStatus::Failed;
            }
            WeightedBudgetBurnPreferenceStatus::Running => {
                return next == WeightedBudgetBurnPreferenceStatus::Done;
            }
            WeightedBudgetBurnPreferenceStatus::Done => {
                return next == WeightedBudgetBurnPreferenceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedBudgetBurnPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted budget burn preference {} cannot move from {} to {}",
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
        format!("{} {}", "weighted budget burn preference", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.rate + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedBudgetBurnPreference {
        WeightedBudgetBurnPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            tenant_id: self.tenant_id.clone(),
            host: self.host.clone(),
            scope: self.scope.clone(),
            ratio: self.ratio,
            rate: self.rate,
            budget: self.budget,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a weighted budget burn preference in status new.
pub fn new_weighted_budget_burn_preference(code: &str, title: &str) -> WeightedBudgetBurnPreference {
    WeightedBudgetBurnPreference {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        tenant_id: "".to_string(),
        host: "".to_string(),
        scope: "".to_string(),
        ratio: 0.0,
        rate: 0.0,
        budget: 0.0,
        floor_value: 0.0,
        status: WeightedBudgetBurnPreferenceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_weighted_budget_burn_preference("code-771", "title-384");
        assert_eq!(item.status, WeightedBudgetBurnPreferenceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_weighted_budget_burn_preference("code-771", "title-384");
        if item.move_to(WeightedBudgetBurnPreferenceStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedBudgetBurnPreferenceStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_weighted_budget_burn_preference("code-771", "title-384");
        item.status = WeightedBudgetBurnPreferenceStatus::Failed;
        assert!(
            item.move_to(WeightedBudgetBurnPreferenceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
