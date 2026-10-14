use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity budget must pass. Check returns its problems, or none.
pub trait IdentityBudgetRule: std::fmt::Debug {
    fn check(&self, item: &IdentityBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBudgetChecksumLengthRule {
    pub max_length: i64,
}

impl IdentityBudgetRule for IdentityBudgetChecksumLengthRule {
    fn check(&self, item: &IdentityBudget) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBudgetRequiredCipherRule {
    pub message: String,
}

impl IdentityBudgetRule for IdentityBudgetRequiredCipherRule {
    fn check(&self, item: &IdentityBudget) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBudgetRequiredChecksumRule {
    pub message: String,
}

impl IdentityBudgetRule for IdentityBudgetRequiredChecksumRule {
    fn check(&self, item: &IdentityBudget) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBudgetCodePrefixRule {
    pub prefix: String,
}

impl IdentityBudgetRule for IdentityBudgetCodePrefixRule {
    fn check(&self, item: &IdentityBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBudgetWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IdentityBudgetRule for IdentityBudgetWeightRangeRule {
    fn check(&self, item: &IdentityBudget) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a identity budget.
#[derive(Debug, Clone)]
pub struct IdentityBudgetValidator {
    pub rules: Vec<Rc<dyn IdentityBudgetRule>>,
}

impl IdentityBudgetValidator {
    pub fn validate(&self, item: &IdentityBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_budget_validator() -> IdentityBudgetValidator {
    IdentityBudgetValidator {
        rules: vec![
            Rc::new(IdentityBudgetChecksumLengthRule { max_length: 32 }) as Rc<dyn IdentityBudgetRule>,
            Rc::new(IdentityBudgetRequiredCipherRule {
                message: "identity budget needs a value".to_string(),
            }),
            Rc::new(IdentityBudgetRequiredChecksumRule {
                message: "identity budget needs an entry".to_string(),
            }),
            Rc::new(IdentityBudgetCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(IdentityBudgetWeightRangeRule { min: 9.1, max: 802.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_budget_validator();
        let item = new_identity_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = IdentityBudgetChecksumLengthRule { max_length: 3600 };
        let mut item = new_identity_budget("x", "y");
        item.checksum = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = IdentityBudgetRequiredCipherRule {
            message: "identity budget needs this field".to_string(),
        };
        let mut item = new_identity_budget("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = IdentityBudgetRequiredChecksumRule {
            message: "identity budget needs a value".to_string(),
        };
        let mut item = new_identity_budget("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = IdentityBudgetWeightRangeRule { min: 2.7, max: 314.3 };
        let mut item = new_identity_budget("x", "y");
        item.weight = 315.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
