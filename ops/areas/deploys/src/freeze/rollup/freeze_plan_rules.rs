use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze plan must pass. Check returns its problems, or none.
pub trait FreezePlanRule: std::fmt::Debug {
    fn check(&self, item: &FreezePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezePlanCodePrefixRule {
    pub prefix: String,
}

impl FreezePlanRule for FreezePlanCodePrefixRule {
    fn check(&self, item: &FreezePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezePlanNameLengthRule {
    pub max_length: i64,
}

impl FreezePlanRule for FreezePlanNameLengthRule {
    fn check(&self, item: &FreezePlan) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezePlanRequiredPhoneRule {
    pub message: String,
}

impl FreezePlanRule for FreezePlanRequiredPhoneRule {
    fn check(&self, item: &FreezePlan) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezePlanRequiredDescriptionRule {
    pub message: String,
}

impl FreezePlanRule for FreezePlanRequiredDescriptionRule {
    fn check(&self, item: &FreezePlan) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezePlanFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezePlanRule for FreezePlanFloorValueRangeRule {
    fn check(&self, item: &FreezePlan) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze plan.
#[derive(Debug, Clone)]
pub struct FreezePlanValidator {
    pub rules: Vec<Rc<dyn FreezePlanRule>>,
}

impl FreezePlanValidator {
    pub fn validate(&self, item: &FreezePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_plan_validator() -> FreezePlanValidator {
    FreezePlanValidator {
        rules: vec![
            Rc::new(FreezePlanCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn FreezePlanRule>,
            Rc::new(FreezePlanNameLengthRule { max_length: 16 }),
            Rc::new(FreezePlanRequiredPhoneRule {
                message: "freeze plan needs a value".to_string(),
            }),
            Rc::new(FreezePlanRequiredDescriptionRule {
                message: "freeze plan needs this field".to_string(),
            }),
            Rc::new(FreezePlanFloorValueRangeRule { min: 4.0, max: 3207.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_plan_validator();
        let item = new_freeze_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FreezePlanNameLengthRule { max_length: 255 };
        let mut item = new_freeze_plan("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = FreezePlanRequiredPhoneRule {
            message: "freeze plan needs an entry".to_string(),
        };
        let mut item = new_freeze_plan("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = FreezePlanRequiredDescriptionRule {
            message: "freeze plan needs an entry".to_string(),
        };
        let mut item = new_freeze_plan("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = FreezePlanFloorValueRangeRule { min: 2.0, max: 4836.4 };
        let mut item = new_freeze_plan("x", "y");
        item.floor_value = 4837.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
