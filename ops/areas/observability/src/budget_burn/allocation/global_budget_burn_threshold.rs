use super::*;

use acme_kit::text;

/// Where a global budget burn threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalBudgetBurnThresholdStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalBudgetBurnThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalBudgetBurnThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global budget burn threshold as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnThreshold {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub ends_at: i64,
    pub weight: f64,
    pub tags: Vec<String>,
    pub burst: i64,
    pub external_ref: String,
    pub active: bool,
    pub status: GlobalBudgetBurnThresholdStatus,
}

impl GlobalBudgetBurnThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalBudgetBurnThresholdStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalBudgetBurnThresholdStatus) -> bool {
        match self.status {
            GlobalBudgetBurnThresholdStatus::Created => {
                match next {
                    GlobalBudgetBurnThresholdStatus::Published | GlobalBudgetBurnThresholdStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalBudgetBurnThresholdStatus::Validated => {
                return next == GlobalBudgetBurnThresholdStatus::Published;
            }
            GlobalBudgetBurnThresholdStatus::Published => {
                return next == GlobalBudgetBurnThresholdStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalBudgetBurnThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global budget burn threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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
        (self.weight * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalBudgetBurnThreshold {
        GlobalBudgetBurnThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            ends_at: self.ends_at,
            weight: self.weight,
            tags: self.tags.clone(),
            burst: self.burst,
            external_ref: self.external_ref.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global budget burn threshold in status created.
pub fn new_global_budget_burn_threshold(code: &str, title: &str) -> GlobalBudgetBurnThreshold {
    GlobalBudgetBurnThreshold {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        ends_at: 0,
        weight: 0.0,
        tags: Vec::new(),
        burst: 0,
        external_ref: "".to_string(),
        active: false,
        status: GlobalBudgetBurnThresholdStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_global_budget_burn_threshold("code-915", "title-857");
        assert_eq!(item.status, GlobalBudgetBurnThresholdStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_global_budget_burn_threshold("code-915", "title-857");
        if item.move_to(GlobalBudgetBurnThresholdStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalBudgetBurnThresholdStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_global_budget_burn_threshold("code-915", "title-857");
        item.status = GlobalBudgetBurnThresholdStatus::Retired;
        assert!(
            item.move_to(GlobalBudgetBurnThresholdStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
