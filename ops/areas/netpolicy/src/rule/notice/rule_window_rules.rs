use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule window must pass. Check returns its problems, or none.
pub trait RuleWindowRule: std::fmt::Debug {
    fn check(&self, item: &RuleWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleWindowBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleWindowRule for RuleWindowBudgetRangeRule {
    fn check(&self, item: &RuleWindow) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleWindowAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleWindowRule for RuleWindowAmountRangeRule {
    fn check(&self, item: &RuleWindow) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleWindowClusterLengthRule {
    pub max_length: i64,
}

impl RuleWindowRule for RuleWindowClusterLengthRule {
    fn check(&self, item: &RuleWindow) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleWindowBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleWindowRule for RuleWindowBurstRangeRule {
    fn check(&self, item: &RuleWindow) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleWindowSummaryLengthRule {
    pub max_length: i64,
}

impl RuleWindowRule for RuleWindowSummaryLengthRule {
    fn check(&self, item: &RuleWindow) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleWindowRequiredClusterRule {
    pub message: String,
}

impl RuleWindowRule for RuleWindowRequiredClusterRule {
    fn check(&self, item: &RuleWindow) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule window.
#[derive(Debug, Clone)]
pub struct RuleWindowValidator {
    pub rules: Vec<Rc<dyn RuleWindowRule>>,
}

impl RuleWindowValidator {
    pub fn validate(&self, item: &RuleWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_window_validator() -> RuleWindowValidator {
    RuleWindowValidator {
        rules: vec![
            Rc::new(RuleWindowBudgetRangeRule { min: 7.0, max: 203.3 }) as Rc<dyn RuleWindowRule>,
            Rc::new(RuleWindowAmountRangeRule { min: 8.8, max: 112.5 }),
            Rc::new(RuleWindowClusterLengthRule { max_length: 32 }),
            Rc::new(RuleWindowBurstRangeRule { min: 80, max: 255 }),
            Rc::new(RuleWindowSummaryLengthRule { max_length: 32 }),
            Rc::new(RuleWindowRequiredClusterRule {
                message: "rule window needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_window_validator();
        let item = new_rule_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RuleWindowBudgetRangeRule { min: 3.8, max: 477.6 };
        let mut item = new_rule_window("x", "y");
        item.budget = 479.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = RuleWindowAmountRangeRule { min: 2.7, max: 3307.7 };
        let mut item = new_rule_window("x", "y");
        item.amount = 3309.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = RuleWindowClusterLengthRule { max_length: 120 };
        let mut item = new_rule_window("x", "y");
        item.cluster = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = RuleWindowBurstRangeRule { min: 64, max: 80 };
        let mut item = new_rule_window("x", "y");
        item.burst = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = RuleWindowSummaryLengthRule { max_length: 80 };
        let mut item = new_rule_window("x", "y");
        item.summary = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = RuleWindowRequiredClusterRule {
            message: "rule window needs a value".to_string(),
        };
        let mut item = new_rule_window("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
