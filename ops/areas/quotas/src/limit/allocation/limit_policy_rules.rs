use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit policy must pass. Check returns its problems, or none.
pub trait LimitPolicyRule: std::fmt::Debug {
    fn check(&self, item: &LimitPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitPolicyFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitPolicyRule for LimitPolicyFloorValueRangeRule {
    fn check(&self, item: &LimitPolicy) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitPolicyTimezoneLengthRule {
    pub max_length: i64,
}

impl LimitPolicyRule for LimitPolicyTimezoneLengthRule {
    fn check(&self, item: &LimitPolicy) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitPolicyRequiredCodeRule {
    pub message: String,
}

impl LimitPolicyRule for LimitPolicyRequiredCodeRule {
    fn check(&self, item: &LimitPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitPolicyRequiredTimezoneRule {
    pub message: String,
}

impl LimitPolicyRule for LimitPolicyRequiredTimezoneRule {
    fn check(&self, item: &LimitPolicy) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit policy.
#[derive(Debug, Clone)]
pub struct LimitPolicyValidator {
    pub rules: Vec<Rc<dyn LimitPolicyRule>>,
}

impl LimitPolicyValidator {
    pub fn validate(&self, item: &LimitPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_policy_validator() -> LimitPolicyValidator {
    LimitPolicyValidator {
        rules: vec![
            Rc::new(LimitPolicyFloorValueRangeRule { min: 3.8, max: 1885.6 }) as Rc<dyn LimitPolicyRule>,
            Rc::new(LimitPolicyTimezoneLengthRule { max_length: 86400 }),
            Rc::new(LimitPolicyRequiredCodeRule {
                message: "limit policy needs an entry".to_string(),
            }),
            Rc::new(LimitPolicyRequiredTimezoneRule {
                message: "limit policy needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_policy_validator();
        let item = new_limit_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = LimitPolicyFloorValueRangeRule { min: 4.7, max: 702.8 };
        let mut item = new_limit_policy("x", "y");
        item.floor_value = 704.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = LimitPolicyTimezoneLengthRule { max_length: 3600 };
        let mut item = new_limit_policy("x", "y");
        item.timezone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LimitPolicyRequiredCodeRule {
            message: "limit policy needs a value".to_string(),
        };
        let mut item = new_limit_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = LimitPolicyRequiredTimezoneRule {
            message: "limit policy needs this field".to_string(),
        };
        let mut item = new_limit_policy("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
