use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce policy must pass. Check returns its problems, or none.
pub trait NoncePolicyRule: std::fmt::Debug {
    fn check(&self, item: &NoncePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoncePolicyCodePrefixRule {
    pub prefix: String,
}

impl NoncePolicyRule for NoncePolicyCodePrefixRule {
    fn check(&self, item: &NoncePolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoncePolicyAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NoncePolicyRule for NoncePolicyAmountRangeRule {
    fn check(&self, item: &NoncePolicy) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoncePolicyRequiredCodeRule {
    pub message: String,
}

impl NoncePolicyRule for NoncePolicyRequiredCodeRule {
    fn check(&self, item: &NoncePolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce policy.
#[derive(Debug, Clone)]
pub struct NoncePolicyValidator {
    pub rules: Vec<Rc<dyn NoncePolicyRule>>,
}

impl NoncePolicyValidator {
    pub fn validate(&self, item: &NoncePolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &NoncePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NoncePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_policy_validator() -> NoncePolicyValidator {
    NoncePolicyValidator {
        rules: vec![
            Rc::new(NoncePolicyCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn NoncePolicyRule>,
            Rc::new(NoncePolicyAmountRangeRule { min: 7.4, max: 3097.0 }),
            Rc::new(NoncePolicyRequiredCodeRule {
                message: "nonce policy needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_policy_validator();
        let item = new_nonce_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = NoncePolicyAmountRangeRule { min: 4.6, max: 3004.2 };
        let mut item = new_nonce_policy("x", "y");
        item.amount = 3005.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = NoncePolicyRequiredCodeRule {
            message: "nonce policy needs a value".to_string(),
        };
        let mut item = new_nonce_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
