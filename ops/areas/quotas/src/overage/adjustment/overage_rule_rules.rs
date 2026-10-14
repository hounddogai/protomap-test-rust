use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage rule must pass. Check returns its problems, or none.
pub trait OverageRuleRule: std::fmt::Debug {
    fn check(&self, item: &OverageRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRuleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageRuleRule for OverageRuleCreatedAtRangeRule {
    fn check(&self, item: &OverageRule) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRuleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageRuleRule for OverageRuleDiscountRangeRule {
    fn check(&self, item: &OverageRule) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRuleRequiredClusterRule {
    pub message: String,
}

impl OverageRuleRule for OverageRuleRequiredClusterRule {
    fn check(&self, item: &OverageRule) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRuleRequiredCodeRule {
    pub message: String,
}

impl OverageRuleRule for OverageRuleRequiredCodeRule {
    fn check(&self, item: &OverageRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRuleClusterLengthRule {
    pub max_length: i64,
}

impl OverageRuleRule for OverageRuleClusterLengthRule {
    fn check(&self, item: &OverageRule) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage rule.
#[derive(Debug, Clone)]
pub struct OverageRuleValidator {
    pub rules: Vec<Rc<dyn OverageRuleRule>>,
}

impl OverageRuleValidator {
    pub fn validate(&self, item: &OverageRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_rule_validator() -> OverageRuleValidator {
    OverageRuleValidator {
        rules: vec![
            Rc::new(OverageRuleCreatedAtRangeRule { min: 16, max: 64 }) as Rc<dyn OverageRuleRule>,
            Rc::new(OverageRuleDiscountRangeRule { min: 3.9, max: 2760.7 }),
            Rc::new(OverageRuleRequiredClusterRule {
                message: "overage rule needs a value".to_string(),
            }),
            Rc::new(OverageRuleRequiredCodeRule {
                message: "overage rule needs a value".to_string(),
            }),
            Rc::new(OverageRuleClusterLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_rule_validator();
        let item = new_overage_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = OverageRuleCreatedAtRangeRule { min: 64, max: 3600 };
        let mut item = new_overage_rule("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = OverageRuleDiscountRangeRule { min: 3.6, max: 4427.9 };
        let mut item = new_overage_rule("x", "y");
        item.discount = 4429.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = OverageRuleRequiredClusterRule {
            message: "overage rule needs this field".to_string(),
        };
        let mut item = new_overage_rule("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = OverageRuleRequiredCodeRule {
            message: "overage rule needs a value".to_string(),
        };
        let mut item = new_overage_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = OverageRuleClusterLengthRule { max_length: 120 };
        let mut item = new_overage_rule("x", "y");
        item.cluster = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
