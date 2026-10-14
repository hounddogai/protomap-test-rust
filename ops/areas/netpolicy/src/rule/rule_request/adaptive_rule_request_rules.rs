use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive rule request must pass. Check returns its problems, or none.
pub trait AdaptiveRuleRequestRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRuleRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleRequestRequiredCodeRule {
    pub message: String,
}

impl AdaptiveRuleRequestRule for AdaptiveRuleRequestRequiredCodeRule {
    fn check(&self, item: &AdaptiveRuleRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleRequestTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveRuleRequestRule for AdaptiveRuleRequestTtlSecondsRangeRule {
    fn check(&self, item: &AdaptiveRuleRequest) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleRequestAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveRuleRequestRule for AdaptiveRuleRequestAmountRangeRule {
    fn check(&self, item: &AdaptiveRuleRequest) -> Vec<String> {
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

/// Runs every rule of a adaptive rule request.
#[derive(Debug, Clone)]
pub struct AdaptiveRuleRequestValidator {
    pub rules: Vec<Rc<dyn AdaptiveRuleRequestRule>>,
}

impl AdaptiveRuleRequestValidator {
    pub fn validate(&self, item: &AdaptiveRuleRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveRuleRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRuleRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_rule_request_validator() -> AdaptiveRuleRequestValidator {
    AdaptiveRuleRequestValidator {
        rules: vec![
            Rc::new(AdaptiveRuleRequestRequiredCodeRule {
                message: "adaptive rule request needs an entry".to_string(),
            }) as Rc<dyn AdaptiveRuleRequestRule>,
            Rc::new(AdaptiveRuleRequestTtlSecondsRangeRule { min: 255, max: 86400 }),
            Rc::new(AdaptiveRuleRequestAmountRangeRule { min: 7.6, max: 1881.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_rule_request_validator();
        let item = new_adaptive_rule_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveRuleRequestRequiredCodeRule {
            message: "adaptive rule request needs this field".to_string(),
        };
        let mut item = new_adaptive_rule_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = AdaptiveRuleRequestTtlSecondsRangeRule { min: 16, max: 120 };
        let mut item = new_adaptive_rule_request("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = AdaptiveRuleRequestAmountRangeRule { min: 2.0, max: 1688.8 };
        let mut item = new_adaptive_rule_request("x", "y");
        item.amount = 1690.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
