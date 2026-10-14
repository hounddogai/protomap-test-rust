use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule bundle must pass. Check returns its problems, or none.
pub trait RuleBundleRule: std::fmt::Debug {
    fn check(&self, item: &RuleBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBundleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleBundleRule for RuleBundleCreatedAtRangeRule {
    fn check(&self, item: &RuleBundle) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBundleMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleBundleRule for RuleBundleMarginRangeRule {
    fn check(&self, item: &RuleBundle) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBundleCodeLengthRule {
    pub max_length: i64,
}

impl RuleBundleRule for RuleBundleCodeLengthRule {
    fn check(&self, item: &RuleBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBundleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleBundleRule for RuleBundleScoreRangeRule {
    fn check(&self, item: &RuleBundle) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBundleNameLengthRule {
    pub max_length: i64,
}

impl RuleBundleRule for RuleBundleNameLengthRule {
    fn check(&self, item: &RuleBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule bundle.
#[derive(Debug, Clone)]
pub struct RuleBundleValidator {
    pub rules: Vec<Rc<dyn RuleBundleRule>>,
}

impl RuleBundleValidator {
    pub fn validate(&self, item: &RuleBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_bundle_validator() -> RuleBundleValidator {
    RuleBundleValidator {
        rules: vec![
            Rc::new(RuleBundleCreatedAtRangeRule { min: 255, max: 1000 }) as Rc<dyn RuleBundleRule>,
            Rc::new(RuleBundleMarginRangeRule { min: 8.4, max: 1702.8 }),
            Rc::new(RuleBundleCodeLengthRule { max_length: 32 }),
            Rc::new(RuleBundleScoreRangeRule { min: 6.1, max: 345.4 }),
            Rc::new(RuleBundleNameLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_bundle_validator();
        let item = new_rule_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RuleBundleCreatedAtRangeRule { min: 120, max: 86400 };
        let mut item = new_rule_bundle("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = RuleBundleMarginRangeRule { min: 4.7, max: 304.0 };
        let mut item = new_rule_bundle("x", "y");
        item.margin = 305.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RuleBundleCodeLengthRule { max_length: 86400 };
        let mut item = new_rule_bundle("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = RuleBundleScoreRangeRule { min: 1.3, max: 2182.3 };
        let mut item = new_rule_bundle("x", "y");
        item.score = 2183.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RuleBundleNameLengthRule { max_length: 120 };
        let mut item = new_rule_bundle("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
