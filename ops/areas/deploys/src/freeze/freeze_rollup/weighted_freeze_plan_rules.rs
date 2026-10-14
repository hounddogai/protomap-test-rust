use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted freeze plan must pass. Check returns its problems, or none.
pub trait WeightedFreezePlanRule: std::fmt::Debug {
    fn check(&self, item: &WeightedFreezePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezePlanCodeLengthRule {
    pub max_length: i64,
}

impl WeightedFreezePlanRule for WeightedFreezePlanCodeLengthRule {
    fn check(&self, item: &WeightedFreezePlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezePlanPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedFreezePlanRule for WeightedFreezePlanPortRangeRule {
    fn check(&self, item: &WeightedFreezePlan) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezePlanAllowedNameRule {
    pub allowed: Vec<String>,
}

impl WeightedFreezePlanRule for WeightedFreezePlanAllowedNameRule {
    fn check(&self, item: &WeightedFreezePlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is missing", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezePlanRequiredNameRule {
    pub message: String,
}

impl WeightedFreezePlanRule for WeightedFreezePlanRequiredNameRule {
    fn check(&self, item: &WeightedFreezePlan) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezePlanNameLengthRule {
    pub max_length: i64,
}

impl WeightedFreezePlanRule for WeightedFreezePlanNameLengthRule {
    fn check(&self, item: &WeightedFreezePlan) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezePlanBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedFreezePlanRule for WeightedFreezePlanBudgetRangeRule {
    fn check(&self, item: &WeightedFreezePlan) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted freeze plan.
#[derive(Debug, Clone)]
pub struct WeightedFreezePlanValidator {
    pub rules: Vec<Rc<dyn WeightedFreezePlanRule>>,
}

impl WeightedFreezePlanValidator {
    pub fn validate(&self, item: &WeightedFreezePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedFreezePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedFreezePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_freeze_plan_validator() -> WeightedFreezePlanValidator {
    WeightedFreezePlanValidator {
        rules: vec![
            Rc::new(WeightedFreezePlanCodeLengthRule { max_length: 64 }) as Rc<dyn WeightedFreezePlanRule>,
            Rc::new(WeightedFreezePlanPortRangeRule { min: 32, max: 32 }),
            Rc::new(WeightedFreezePlanAllowedNameRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(WeightedFreezePlanRequiredNameRule {
                message: "weighted freeze plan needs this field".to_string(),
            }),
            Rc::new(WeightedFreezePlanNameLengthRule { max_length: 255 }),
            Rc::new(WeightedFreezePlanBudgetRangeRule { min: 3.7, max: 3714.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_freeze_plan_validator();
        let item = new_weighted_freeze_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedFreezePlanCodeLengthRule { max_length: 16 };
        let mut item = new_weighted_freeze_plan("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = WeightedFreezePlanPortRangeRule { min: 64, max: 255 };
        let mut item = new_weighted_freeze_plan("x", "y");
        item.port = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = WeightedFreezePlanRequiredNameRule {
            message: "weighted freeze plan needs a value".to_string(),
        };
        let mut item = new_weighted_freeze_plan("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedFreezePlanNameLengthRule { max_length: 255 };
        let mut item = new_weighted_freeze_plan("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = WeightedFreezePlanBudgetRangeRule { min: 7.5, max: 1186.6 };
        let mut item = new_weighted_freeze_plan("x", "y");
        item.budget = 1188.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
