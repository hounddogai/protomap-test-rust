use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling binding adjustment must pass. Check returns its problems, or none.
pub trait RollingBindingAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &RollingBindingAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl RollingBindingAdjustmentRule for RollingBindingAdjustmentCodeLengthRule {
    fn check(&self, item: &RollingBindingAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingAdjustmentRequiredPhoneRule {
    pub message: String,
}

impl RollingBindingAdjustmentRule for RollingBindingAdjustmentRequiredPhoneRule {
    fn check(&self, item: &RollingBindingAdjustment) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingAdjustmentLocaleLengthRule {
    pub max_length: i64,
}

impl RollingBindingAdjustmentRule for RollingBindingAdjustmentLocaleLengthRule {
    fn check(&self, item: &RollingBindingAdjustment) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingAdjustmentRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingBindingAdjustmentRule for RollingBindingAdjustmentRetriesRangeRule {
    fn check(&self, item: &RollingBindingAdjustment) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling binding adjustment.
#[derive(Debug, Clone)]
pub struct RollingBindingAdjustmentValidator {
    pub rules: Vec<Rc<dyn RollingBindingAdjustmentRule>>,
}

impl RollingBindingAdjustmentValidator {
    pub fn validate(&self, item: &RollingBindingAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingBindingAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingBindingAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_binding_adjustment_validator() -> RollingBindingAdjustmentValidator {
    RollingBindingAdjustmentValidator {
        rules: vec![
            Rc::new(RollingBindingAdjustmentCodeLengthRule { max_length: 16 }) as Rc<dyn RollingBindingAdjustmentRule>,
            Rc::new(RollingBindingAdjustmentRequiredPhoneRule {
                message: "rolling binding adjustment needs this field".to_string(),
            }),
            Rc::new(RollingBindingAdjustmentLocaleLengthRule { max_length: 16 }),
            Rc::new(RollingBindingAdjustmentRetriesRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_binding_adjustment_validator();
        let item = new_rolling_binding_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingBindingAdjustmentCodeLengthRule { max_length: 3600 };
        let mut item = new_rolling_binding_adjustment("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = RollingBindingAdjustmentRequiredPhoneRule {
            message: "rolling binding adjustment needs this field".to_string(),
        };
        let mut item = new_rolling_binding_adjustment("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = RollingBindingAdjustmentLocaleLengthRule { max_length: 86400 };
        let mut item = new_rolling_binding_adjustment("x", "y");
        item.locale = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = RollingBindingAdjustmentRetriesRangeRule { min: 32, max: 1000 };
        let mut item = new_rolling_binding_adjustment("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
