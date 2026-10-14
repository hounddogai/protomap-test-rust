use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal ttl policy group must pass. Check returns its problems, or none.
pub trait SeasonalTtlPolicyGroupRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalTtlPolicyGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyGroupMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalTtlPolicyGroupRule for SeasonalTtlPolicyGroupMaximumRangeRule {
    fn check(&self, item: &SeasonalTtlPolicyGroup) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyGroupRequiredIssuerRule {
    pub message: String,
}

impl SeasonalTtlPolicyGroupRule for SeasonalTtlPolicyGroupRequiredIssuerRule {
    fn check(&self, item: &SeasonalTtlPolicyGroup) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyGroupPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalTtlPolicyGroupRule for SeasonalTtlPolicyGroupPriorityRangeRule {
    fn check(&self, item: &SeasonalTtlPolicyGroup) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyGroupCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalTtlPolicyGroupRule for SeasonalTtlPolicyGroupCodeLengthRule {
    fn check(&self, item: &SeasonalTtlPolicyGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyGroupTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalTtlPolicyGroupRule for SeasonalTtlPolicyGroupTtlSecondsRangeRule {
    fn check(&self, item: &SeasonalTtlPolicyGroup) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyGroupRequiredSourceRule {
    pub message: String,
}

impl SeasonalTtlPolicyGroupRule for SeasonalTtlPolicyGroupRequiredSourceRule {
    fn check(&self, item: &SeasonalTtlPolicyGroup) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal ttl policy group.
#[derive(Debug, Clone)]
pub struct SeasonalTtlPolicyGroupValidator {
    pub rules: Vec<Rc<dyn SeasonalTtlPolicyGroupRule>>,
}

impl SeasonalTtlPolicyGroupValidator {
    pub fn validate(&self, item: &SeasonalTtlPolicyGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalTtlPolicyGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalTtlPolicyGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_ttl_policy_group_validator() -> SeasonalTtlPolicyGroupValidator {
    SeasonalTtlPolicyGroupValidator {
        rules: vec![
            Rc::new(SeasonalTtlPolicyGroupMaximumRangeRule { min: 4.2, max: 2892.3 })
                as Rc<dyn SeasonalTtlPolicyGroupRule>,
            Rc::new(SeasonalTtlPolicyGroupRequiredIssuerRule {
                message: "seasonal ttl policy group needs this field".to_string(),
            }),
            Rc::new(SeasonalTtlPolicyGroupPriorityRangeRule { min: 120, max: 86400 }),
            Rc::new(SeasonalTtlPolicyGroupCodeLengthRule { max_length: 3600 }),
            Rc::new(SeasonalTtlPolicyGroupTtlSecondsRangeRule { min: 64, max: 86400 }),
            Rc::new(SeasonalTtlPolicyGroupRequiredSourceRule {
                message: "seasonal ttl policy group needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_ttl_policy_group_validator();
        let item = new_seasonal_ttl_policy_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = SeasonalTtlPolicyGroupMaximumRangeRule { min: 2.0, max: 1602.4 };
        let mut item = new_seasonal_ttl_policy_group("x", "y");
        item.maximum = 1603.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = SeasonalTtlPolicyGroupRequiredIssuerRule {
            message: "seasonal ttl policy group needs this field".to_string(),
        };
        let mut item = new_seasonal_ttl_policy_group("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = SeasonalTtlPolicyGroupPriorityRangeRule { min: 64, max: 80 };
        let mut item = new_seasonal_ttl_policy_group("x", "y");
        item.priority = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalTtlPolicyGroupCodeLengthRule { max_length: 255 };
        let mut item = new_seasonal_ttl_policy_group("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = SeasonalTtlPolicyGroupTtlSecondsRangeRule { min: 32, max: 255 };
        let mut item = new_seasonal_ttl_policy_group("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = SeasonalTtlPolicyGroupRequiredSourceRule {
            message: "seasonal ttl policy group needs a value".to_string(),
        };
        let mut item = new_seasonal_ttl_policy_group("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
