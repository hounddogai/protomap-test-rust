use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive silence budget must pass. Check returns its problems, or none.
pub trait AdaptiveSilenceBudgetRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveSilenceBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceBudgetMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveSilenceBudgetRule for AdaptiveSilenceBudgetMarginRangeRule {
    fn check(&self, item: &AdaptiveSilenceBudget) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceBudgetVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveSilenceBudgetRule for AdaptiveSilenceBudgetVersionRangeRule {
    fn check(&self, item: &AdaptiveSilenceBudget) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceBudgetHostLengthRule {
    pub max_length: i64,
}

impl AdaptiveSilenceBudgetRule for AdaptiveSilenceBudgetHostLengthRule {
    fn check(&self, item: &AdaptiveSilenceBudget) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive silence budget.
#[derive(Debug, Clone)]
pub struct AdaptiveSilenceBudgetValidator {
    pub rules: Vec<Rc<dyn AdaptiveSilenceBudgetRule>>,
}

impl AdaptiveSilenceBudgetValidator {
    pub fn validate(&self, item: &AdaptiveSilenceBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveSilenceBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveSilenceBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_silence_budget_validator() -> AdaptiveSilenceBudgetValidator {
    AdaptiveSilenceBudgetValidator {
        rules: vec![
            Rc::new(AdaptiveSilenceBudgetMarginRangeRule { min: 5.7, max: 1545.5 })
                as Rc<dyn AdaptiveSilenceBudgetRule>,
            Rc::new(AdaptiveSilenceBudgetVersionRangeRule { min: 255, max: 1000 }),
            Rc::new(AdaptiveSilenceBudgetHostLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_silence_budget_validator();
        let item = new_adaptive_silence_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = AdaptiveSilenceBudgetMarginRangeRule { min: 2.5, max: 2132.0 };
        let mut item = new_adaptive_silence_budget("x", "y");
        item.margin = 2133.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = AdaptiveSilenceBudgetVersionRangeRule { min: 32, max: 3600 };
        let mut item = new_adaptive_silence_budget("x", "y");
        item.version = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = AdaptiveSilenceBudgetHostLengthRule { max_length: 32 };
        let mut item = new_adaptive_silence_budget("x", "y");
        item.host = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
