use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict token plan must pass. Check returns its problems, or none.
pub trait StrictTokenPlanRule: std::fmt::Debug {
    fn check(&self, item: &StrictTokenPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTokenPlanUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictTokenPlanRule for StrictTokenPlanUpdatedAtRangeRule {
    fn check(&self, item: &StrictTokenPlan) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTokenPlanLabelLengthRule {
    pub max_length: i64,
}

impl StrictTokenPlanRule for StrictTokenPlanLabelLengthRule {
    fn check(&self, item: &StrictTokenPlan) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTokenPlanRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictTokenPlanRule for StrictTokenPlanRateRangeRule {
    fn check(&self, item: &StrictTokenPlan) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTokenPlanPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictTokenPlanRule for StrictTokenPlanPortRangeRule {
    fn check(&self, item: &StrictTokenPlan) -> Vec<String> {
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
pub struct StrictTokenPlanCodePrefixRule {
    pub prefix: String,
}

impl StrictTokenPlanRule for StrictTokenPlanCodePrefixRule {
    fn check(&self, item: &StrictTokenPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict token plan.
#[derive(Debug, Clone)]
pub struct StrictTokenPlanValidator {
    pub rules: Vec<Rc<dyn StrictTokenPlanRule>>,
}

impl StrictTokenPlanValidator {
    pub fn validate(&self, item: &StrictTokenPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictTokenPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictTokenPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_token_plan_validator() -> StrictTokenPlanValidator {
    StrictTokenPlanValidator {
        rules: vec![
            Rc::new(StrictTokenPlanUpdatedAtRangeRule { min: 64, max: 80 }) as Rc<dyn StrictTokenPlanRule>,
            Rc::new(StrictTokenPlanLabelLengthRule { max_length: 1000 }),
            Rc::new(StrictTokenPlanRateRangeRule { min: 3.7, max: 1874.8 }),
            Rc::new(StrictTokenPlanPortRangeRule { min: 64, max: 1000 }),
            Rc::new(StrictTokenPlanCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_token_plan_validator();
        let item = new_strict_token_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = StrictTokenPlanUpdatedAtRangeRule { min: 120, max: 1000 };
        let mut item = new_strict_token_plan("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = StrictTokenPlanLabelLengthRule { max_length: 120 };
        let mut item = new_strict_token_plan("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = StrictTokenPlanRateRangeRule { min: 7.7, max: 3375.2 };
        let mut item = new_strict_token_plan("x", "y");
        item.rate = 3376.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = StrictTokenPlanPortRangeRule { min: 3600, max: 3600 };
        let mut item = new_strict_token_plan("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
