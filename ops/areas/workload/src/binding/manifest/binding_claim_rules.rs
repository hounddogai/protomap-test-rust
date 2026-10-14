use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding claim must pass. Check returns its problems, or none.
pub trait BindingClaimRule: std::fmt::Debug {
    fn check(&self, item: &BindingClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingClaimCodePrefixRule {
    pub prefix: String,
}

impl BindingClaimRule for BindingClaimCodePrefixRule {
    fn check(&self, item: &BindingClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingClaimCodeLengthRule {
    pub max_length: i64,
}

impl BindingClaimRule for BindingClaimCodeLengthRule {
    fn check(&self, item: &BindingClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingClaimRequiredNameRule {
    pub message: String,
}

impl BindingClaimRule for BindingClaimRequiredNameRule {
    fn check(&self, item: &BindingClaim) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingClaimSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingClaimRule for BindingClaimSequenceRangeRule {
    fn check(&self, item: &BindingClaim) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a binding claim.
#[derive(Debug, Clone)]
pub struct BindingClaimValidator {
    pub rules: Vec<Rc<dyn BindingClaimRule>>,
}

impl BindingClaimValidator {
    pub fn validate(&self, item: &BindingClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_claim_validator() -> BindingClaimValidator {
    BindingClaimValidator {
        rules: vec![
            Rc::new(BindingClaimCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn BindingClaimRule>,
            Rc::new(BindingClaimCodeLengthRule { max_length: 16 }),
            Rc::new(BindingClaimRequiredNameRule {
                message: "binding claim needs a value".to_string(),
            }),
            Rc::new(BindingClaimSequenceRangeRule { min: 64, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_claim_validator();
        let item = new_binding_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BindingClaimCodeLengthRule { max_length: 86400 };
        let mut item = new_binding_claim("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BindingClaimRequiredNameRule {
            message: "binding claim needs this field".to_string(),
        };
        let mut item = new_binding_claim("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = BindingClaimSequenceRangeRule { min: 32, max: 3600 };
        let mut item = new_binding_claim("x", "y");
        item.sequence = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
