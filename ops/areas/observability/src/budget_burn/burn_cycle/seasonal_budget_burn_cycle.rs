use super::*;

use acme_kit::text;

/// Where a seasonal budget burn cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalBudgetBurnCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalBudgetBurnCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalBudgetBurnCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal budget burn cycle as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnCycle {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub cipher: String,
    pub unit: String,
    pub archived: bool,
    pub visible: bool,
    pub summary: String,
    pub fingerprint: String,
    pub status: SeasonalBudgetBurnCycleStatus,
}

impl SeasonalBudgetBurnCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalBudgetBurnCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalBudgetBurnCycleStatus) -> bool {
        match self.status {
            SeasonalBudgetBurnCycleStatus::Created => {
                return next == SeasonalBudgetBurnCycleStatus::Published;
            }
            SeasonalBudgetBurnCycleStatus::Validated => {
                match next {
                    SeasonalBudgetBurnCycleStatus::Published | SeasonalBudgetBurnCycleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalBudgetBurnCycleStatus::Published => {
                return next == SeasonalBudgetBurnCycleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalBudgetBurnCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal budget burn cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal budget burn cycle", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> SeasonalBudgetBurnCycle {
        SeasonalBudgetBurnCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            cipher: self.cipher.clone(),
            unit: self.unit.clone(),
            archived: self.archived,
            visible: self.visible,
            summary: self.summary.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a seasonal budget burn cycle in status created.
pub fn new_seasonal_budget_burn_cycle(code: &str, name: &str) -> SeasonalBudgetBurnCycle {
    SeasonalBudgetBurnCycle {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        cipher: "".to_string(),
        unit: "".to_string(),
        archived: false,
        visible: false,
        summary: "".to_string(),
        fingerprint: "".to_string(),
        status: SeasonalBudgetBurnCycleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_seasonal_budget_burn_cycle("code-582", "name-468");
        assert_eq!(item.status, SeasonalBudgetBurnCycleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_seasonal_budget_burn_cycle("code-582", "name-468");
        if item.move_to(SeasonalBudgetBurnCycleStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalBudgetBurnCycleStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_seasonal_budget_burn_cycle("code-582", "name-468");
        item.status = SeasonalBudgetBurnCycleStatus::Retired;
        assert!(
            item.move_to(SeasonalBudgetBurnCycleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
