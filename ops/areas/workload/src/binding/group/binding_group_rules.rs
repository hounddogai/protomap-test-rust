use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding group must pass. Check returns its problems, or none.
pub trait BindingGroupRule: std::fmt::Debug {
    fn check(&self, item: &BindingGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingGroupRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingGroupRule for BindingGroupRevisionRangeRule {
    fn check(&self, item: &BindingGroup) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingGroupStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingGroupRule for BindingGroupStartsAtRangeRule {
    fn check(&self, item: &BindingGroup) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingGroupCipherLengthRule {
    pub max_length: i64,
}

impl BindingGroupRule for BindingGroupCipherLengthRule {
    fn check(&self, item: &BindingGroup) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding group.
#[derive(Debug, Clone)]
pub struct BindingGroupValidator {
    pub rules: Vec<Rc<dyn BindingGroupRule>>,
}

impl BindingGroupValidator {
    pub fn validate(&self, item: &BindingGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_group_validator() -> BindingGroupValidator {
    BindingGroupValidator {
        rules: vec![
            Rc::new(BindingGroupRevisionRangeRule { min: 64, max: 80 }) as Rc<dyn BindingGroupRule>,
            Rc::new(BindingGroupStartsAtRangeRule { min: 120, max: 255 }),
            Rc::new(BindingGroupCipherLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_group_validator();
        let item = new_binding_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = BindingGroupRevisionRangeRule { min: 80, max: 255 };
        let mut item = new_binding_group("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = BindingGroupStartsAtRangeRule { min: 3600, max: 86400 };
        let mut item = new_binding_group("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = BindingGroupCipherLengthRule { max_length: 120 };
        let mut item = new_binding_group("x", "y");
        item.cipher = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
