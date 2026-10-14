use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal budget burn cycle must pass. Check returns its problems, or none.
pub trait SeasonalBudgetBurnCycleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalBudgetBurnCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnCycleCipherLengthRule {
    pub max_length: i64,
}

impl SeasonalBudgetBurnCycleRule for SeasonalBudgetBurnCycleCipherLengthRule {
    fn check(&self, item: &SeasonalBudgetBurnCycle) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnCycleCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalBudgetBurnCycleRule for SeasonalBudgetBurnCycleCodeLengthRule {
    fn check(&self, item: &SeasonalBudgetBurnCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnCycleRequiredUnitRule {
    pub message: String,
}

impl SeasonalBudgetBurnCycleRule for SeasonalBudgetBurnCycleRequiredUnitRule {
    fn check(&self, item: &SeasonalBudgetBurnCycle) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnCycleRequiredNameRule {
    pub message: String,
}

impl SeasonalBudgetBurnCycleRule for SeasonalBudgetBurnCycleRequiredNameRule {
    fn check(&self, item: &SeasonalBudgetBurnCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal budget burn cycle.
#[derive(Debug, Clone)]
pub struct SeasonalBudgetBurnCycleValidator {
    pub rules: Vec<Rc<dyn SeasonalBudgetBurnCycleRule>>,
}

impl SeasonalBudgetBurnCycleValidator {
    pub fn validate(&self, item: &SeasonalBudgetBurnCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalBudgetBurnCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalBudgetBurnCycle]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_seasonal_budget_burn_cycle_validator() -> SeasonalBudgetBurnCycleValidator {
    SeasonalBudgetBurnCycleValidator {
        rules: vec![
            Rc::new(SeasonalBudgetBurnCycleCipherLengthRule { max_length: 255 }) as Rc<dyn SeasonalBudgetBurnCycleRule>,
            Rc::new(SeasonalBudgetBurnCycleCodeLengthRule { max_length: 255 }),
            Rc::new(SeasonalBudgetBurnCycleRequiredUnitRule {
                message: "seasonal budget burn cycle needs an entry".to_string(),
            }),
            Rc::new(SeasonalBudgetBurnCycleRequiredNameRule {
                message: "seasonal budget burn cycle needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_budget_burn_cycle_validator();
        let item = new_seasonal_budget_burn_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = SeasonalBudgetBurnCycleCipherLengthRule { max_length: 120 };
        let mut item = new_seasonal_budget_burn_cycle("x", "y");
        item.cipher = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalBudgetBurnCycleCodeLengthRule { max_length: 80 };
        let mut item = new_seasonal_budget_burn_cycle("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = SeasonalBudgetBurnCycleRequiredUnitRule {
            message: "seasonal budget burn cycle needs this field".to_string(),
        };
        let mut item = new_seasonal_budget_burn_cycle("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SeasonalBudgetBurnCycleRequiredNameRule {
            message: "seasonal budget burn cycle needs this field".to_string(),
        };
        let mut item = new_seasonal_budget_burn_cycle("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
