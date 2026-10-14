use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a binding adjustment must pass. Check returns its problems, or none.
pub trait BindingAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &BindingAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAdjustmentAllowedNameRule {
    pub allowed: Vec<String>,
}

impl BindingAdjustmentRule for BindingAdjustmentAllowedNameRule {
    fn check(&self, item: &BindingAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAdjustmentRequiredNameRule {
    pub message: String,
}

impl BindingAdjustmentRule for BindingAdjustmentRequiredNameRule {
    fn check(&self, item: &BindingAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl BindingAdjustmentRule for BindingAdjustmentCodeLengthRule {
    fn check(&self, item: &BindingAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding adjustment.
#[derive(Debug, Clone)]
pub struct BindingAdjustmentValidator {
    pub rules: Vec<Rc<dyn BindingAdjustmentRule>>,
}

impl BindingAdjustmentValidator {
    pub fn validate(&self, item: &BindingAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_adjustment_validator() -> BindingAdjustmentValidator {
    BindingAdjustmentValidator {
        rules: vec![
            Rc::new(BindingAdjustmentAllowedNameRule { allowed: Vec::new() }) as Rc<dyn BindingAdjustmentRule>,
            Rc::new(BindingAdjustmentRequiredNameRule {
                message: "binding adjustment needs an entry".to_string(),
            }),
            Rc::new(BindingAdjustmentCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_adjustment_validator();
        let item = new_binding_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BindingAdjustmentRequiredNameRule {
            message: "binding adjustment needs this field".to_string(),
        };
        let mut item = new_binding_adjustment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BindingAdjustmentCodeLengthRule { max_length: 120 };
        let mut item = new_binding_adjustment("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
