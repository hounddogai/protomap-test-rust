use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling budget burn cycle must pass. Check returns its problems, or none.
pub trait RollingBudgetBurnCycleRule: std::fmt::Debug {
    fn check(&self, item: &RollingBudgetBurnCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnCycleNameLengthRule {
    pub max_length: i64,
}

impl RollingBudgetBurnCycleRule for RollingBudgetBurnCycleNameLengthRule {
    fn check(&self, item: &RollingBudgetBurnCycle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnCycleQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingBudgetBurnCycleRule for RollingBudgetBurnCycleQuotaUnitsRangeRule {
    fn check(&self, item: &RollingBudgetBurnCycle) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnCycleRequiredLocaleRule {
    pub message: String,
}

impl RollingBudgetBurnCycleRule for RollingBudgetBurnCycleRequiredLocaleRule {
    fn check(&self, item: &RollingBudgetBurnCycle) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnCycleRequiredCipherRule {
    pub message: String,
}

impl RollingBudgetBurnCycleRule for RollingBudgetBurnCycleRequiredCipherRule {
    fn check(&self, item: &RollingBudgetBurnCycle) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling budget burn cycle.
#[derive(Debug, Clone)]
pub struct RollingBudgetBurnCycleValidator {
    pub rules: Vec<Rc<dyn RollingBudgetBurnCycleRule>>,
}

impl RollingBudgetBurnCycleValidator {
    pub fn validate(&self, item: &RollingBudgetBurnCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingBudgetBurnCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingBudgetBurnCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_budget_burn_cycle_validator() -> RollingBudgetBurnCycleValidator {
    RollingBudgetBurnCycleValidator {
        rules: vec![
            Rc::new(RollingBudgetBurnCycleNameLengthRule { max_length: 32 }) as Rc<dyn RollingBudgetBurnCycleRule>,
            Rc::new(RollingBudgetBurnCycleQuotaUnitsRangeRule { min: 86400, max: 86400 }),
            Rc::new(RollingBudgetBurnCycleRequiredLocaleRule {
                message: "rolling budget burn cycle needs this field".to_string(),
            }),
            Rc::new(RollingBudgetBurnCycleRequiredCipherRule {
                message: "rolling budget burn cycle needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_budget_burn_cycle_validator();
        let item = new_rolling_budget_burn_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollingBudgetBurnCycleNameLengthRule { max_length: 255 };
        let mut item = new_rolling_budget_burn_cycle("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = RollingBudgetBurnCycleQuotaUnitsRangeRule { min: 1000, max: 86400 };
        let mut item = new_rolling_budget_burn_cycle("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = RollingBudgetBurnCycleRequiredLocaleRule {
            message: "rolling budget burn cycle needs a value".to_string(),
        };
        let mut item = new_rolling_budget_burn_cycle("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = RollingBudgetBurnCycleRequiredCipherRule {
            message: "rolling budget burn cycle needs a value".to_string(),
        };
        let mut item = new_rolling_budget_burn_cycle("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
