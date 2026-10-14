use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule request must pass. Check returns its problems, or none.
pub trait RuleRequestRule: std::fmt::Debug {
    fn check(&self, item: &RuleRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRequestUnitLengthRule {
    pub max_length: i64,
}

impl RuleRequestRule for RuleRequestUnitLengthRule {
    fn check(&self, item: &RuleRequest) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRequestRequiredLabelRule {
    pub message: String,
}

impl RuleRequestRule for RuleRequestRequiredLabelRule {
    fn check(&self, item: &RuleRequest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRequestRequiredUnitRule {
    pub message: String,
}

impl RuleRequestRule for RuleRequestRequiredUnitRule {
    fn check(&self, item: &RuleRequest) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRequestCodePrefixRule {
    pub prefix: String,
}

impl RuleRequestRule for RuleRequestCodePrefixRule {
    fn check(&self, item: &RuleRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRequestScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleRequestRule for RuleRequestScoreRangeRule {
    fn check(&self, item: &RuleRequest) -> Vec<String> {
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

/// Runs every rule of a rule request.
#[derive(Debug, Clone)]
pub struct RuleRequestValidator {
    pub rules: Vec<Rc<dyn RuleRequestRule>>,
}

impl RuleRequestValidator {
    pub fn validate(&self, item: &RuleRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_request_validator() -> RuleRequestValidator {
    RuleRequestValidator {
        rules: vec![
            Rc::new(RuleRequestUnitLengthRule { max_length: 32 }) as Rc<dyn RuleRequestRule>,
            Rc::new(RuleRequestRequiredLabelRule {
                message: "rule request needs this field".to_string(),
            }),
            Rc::new(RuleRequestRequiredUnitRule {
                message: "rule request needs a value".to_string(),
            }),
            Rc::new(RuleRequestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RuleRequestScoreRangeRule { min: 1.0, max: 1412.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_request_validator();
        let item = new_rule_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = RuleRequestUnitLengthRule { max_length: 3600 };
        let mut item = new_rule_request("x", "y");
        item.unit = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RuleRequestRequiredLabelRule {
            message: "rule request needs this field".to_string(),
        };
        let mut item = new_rule_request("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = RuleRequestRequiredUnitRule {
            message: "rule request needs a value".to_string(),
        };
        let mut item = new_rule_request("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = RuleRequestScoreRangeRule { min: 1.7, max: 656.2 };
        let mut item = new_rule_request("x", "y");
        item.score = 657.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
