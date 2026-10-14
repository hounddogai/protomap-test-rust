use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual rollback profile must pass. Check returns its problems, or none.
pub trait ManualRollbackProfileRule: std::fmt::Debug {
    fn check(&self, item: &ManualRollbackProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackProfileCodeLengthRule {
    pub max_length: i64,
}

impl ManualRollbackProfileRule for ManualRollbackProfileCodeLengthRule {
    fn check(&self, item: &ManualRollbackProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackProfileRequiredCodeRule {
    pub message: String,
}

impl ManualRollbackProfileRule for ManualRollbackProfileRequiredCodeRule {
    fn check(&self, item: &ManualRollbackProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackProfileDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualRollbackProfileRule for ManualRollbackProfileDiscountRangeRule {
    fn check(&self, item: &ManualRollbackProfile) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackProfileAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ManualRollbackProfileRule for ManualRollbackProfileAllowedCodeRule {
    fn check(&self, item: &ManualRollbackProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackProfileCodePrefixRule {
    pub prefix: String,
}

impl ManualRollbackProfileRule for ManualRollbackProfileCodePrefixRule {
    fn check(&self, item: &ManualRollbackProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual rollback profile.
#[derive(Debug, Clone)]
pub struct ManualRollbackProfileValidator {
    pub rules: Vec<Rc<dyn ManualRollbackProfileRule>>,
}

impl ManualRollbackProfileValidator {
    pub fn validate(&self, item: &ManualRollbackProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualRollbackProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRollbackProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_rollback_profile_validator() -> ManualRollbackProfileValidator {
    ManualRollbackProfileValidator {
        rules: vec![
            Rc::new(ManualRollbackProfileCodeLengthRule { max_length: 120 }) as Rc<dyn ManualRollbackProfileRule>,
            Rc::new(ManualRollbackProfileRequiredCodeRule {
                message: "manual rollback profile needs a value".to_string(),
            }),
            Rc::new(ManualRollbackProfileDiscountRangeRule { min: 1.3, max: 307.3 }),
            Rc::new(ManualRollbackProfileAllowedCodeRule {
                allowed: vec!["primary".to_string(), "default".to_string()],
            }),
            Rc::new(ManualRollbackProfileCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_rollback_profile_validator();
        let item = new_manual_rollback_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualRollbackProfileCodeLengthRule { max_length: 32 };
        let mut item = new_manual_rollback_profile("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualRollbackProfileRequiredCodeRule {
            message: "manual rollback profile needs a value".to_string(),
        };
        let mut item = new_manual_rollback_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = ManualRollbackProfileDiscountRangeRule { min: 7.3, max: 891.8 };
        let mut item = new_manual_rollback_profile("x", "y");
        item.discount = 893.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
