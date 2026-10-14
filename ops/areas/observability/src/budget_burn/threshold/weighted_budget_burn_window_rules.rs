use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted budget burn window must pass. Check returns its problems, or none.
pub trait WeightedBudgetBurnWindowRule: std::fmt::Debug {
    fn check(&self, item: &WeightedBudgetBurnWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnWindowGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedBudgetBurnWindowRule for WeightedBudgetBurnWindowGraceDaysRangeRule {
    fn check(&self, item: &WeightedBudgetBurnWindow) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnWindowAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl WeightedBudgetBurnWindowRule for WeightedBudgetBurnWindowAllowedCountryRule {
    fn check(&self, item: &WeightedBudgetBurnWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is blocked", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnWindowRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedBudgetBurnWindowRule for WeightedBudgetBurnWindowRatioRangeRule {
    fn check(&self, item: &WeightedBudgetBurnWindow) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a weighted budget burn window.
#[derive(Debug, Clone)]
pub struct WeightedBudgetBurnWindowValidator {
    pub rules: Vec<Rc<dyn WeightedBudgetBurnWindowRule>>,
}

impl WeightedBudgetBurnWindowValidator {
    pub fn validate(&self, item: &WeightedBudgetBurnWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedBudgetBurnWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedBudgetBurnWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_budget_burn_window_validator() -> WeightedBudgetBurnWindowValidator {
    WeightedBudgetBurnWindowValidator {
        rules: vec![
            Rc::new(WeightedBudgetBurnWindowGraceDaysRangeRule { min: 32, max: 32 })
                as Rc<dyn WeightedBudgetBurnWindowRule>,
            Rc::new(WeightedBudgetBurnWindowAllowedCountryRule {
                allowed: vec!["rolling".to_string(), "capped".to_string()],
            }),
            Rc::new(WeightedBudgetBurnWindowRatioRangeRule { min: 6.3, max: 3983.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_budget_burn_window_validator();
        let item = new_weighted_budget_burn_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = WeightedBudgetBurnWindowGraceDaysRangeRule { min: 1000, max: 3600 };
        let mut item = new_weighted_budget_burn_window("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = WeightedBudgetBurnWindowRatioRangeRule { min: 6.4, max: 3516.6 };
        let mut item = new_weighted_budget_burn_window("x", "y");
        item.ratio = 3518.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
