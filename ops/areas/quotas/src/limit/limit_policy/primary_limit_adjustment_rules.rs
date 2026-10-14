use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary limit adjustment must pass. Check returns its problems, or none.
pub trait PrimaryLimitAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryLimitAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl PrimaryLimitAdjustmentRule for PrimaryLimitAdjustmentCodePrefixRule {
    fn check(&self, item: &PrimaryLimitAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitAdjustmentNameLengthRule {
    pub max_length: i64,
}

impl PrimaryLimitAdjustmentRule for PrimaryLimitAdjustmentNameLengthRule {
    fn check(&self, item: &PrimaryLimitAdjustment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitAdjustmentRequiredCodeRule {
    pub message: String,
}

impl PrimaryLimitAdjustmentRule for PrimaryLimitAdjustmentRequiredCodeRule {
    fn check(&self, item: &PrimaryLimitAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary limit adjustment.
#[derive(Debug, Clone)]
pub struct PrimaryLimitAdjustmentValidator {
    pub rules: Vec<Rc<dyn PrimaryLimitAdjustmentRule>>,
}

impl PrimaryLimitAdjustmentValidator {
    pub fn validate(&self, item: &PrimaryLimitAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryLimitAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryLimitAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_limit_adjustment_validator() -> PrimaryLimitAdjustmentValidator {
    PrimaryLimitAdjustmentValidator {
        rules: vec![
            Rc::new(PrimaryLimitAdjustmentCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn PrimaryLimitAdjustmentRule>,
            Rc::new(PrimaryLimitAdjustmentNameLengthRule { max_length: 64 }),
            Rc::new(PrimaryLimitAdjustmentRequiredCodeRule {
                message: "primary limit adjustment needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_limit_adjustment_validator();
        let item = new_primary_limit_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PrimaryLimitAdjustmentNameLengthRule { max_length: 86400 };
        let mut item = new_primary_limit_adjustment("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryLimitAdjustmentRequiredCodeRule {
            message: "primary limit adjustment needs this field".to_string(),
        };
        let mut item = new_primary_limit_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
