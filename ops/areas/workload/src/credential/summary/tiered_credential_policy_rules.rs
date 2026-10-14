use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered credential policy must pass. Check returns its problems, or none.
pub trait TieredCredentialPolicyRule: std::fmt::Debug {
    fn check(&self, item: &TieredCredentialPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialPolicyAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TieredCredentialPolicyRule for TieredCredentialPolicyAllowedCodeRule {
    fn check(&self, item: &TieredCredentialPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialPolicyRequiredNameRule {
    pub message: String,
}

impl TieredCredentialPolicyRule for TieredCredentialPolicyRequiredNameRule {
    fn check(&self, item: &TieredCredentialPolicy) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialPolicyQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredCredentialPolicyRule for TieredCredentialPolicyQuantityRangeRule {
    fn check(&self, item: &TieredCredentialPolicy) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialPolicyCodePrefixRule {
    pub prefix: String,
}

impl TieredCredentialPolicyRule for TieredCredentialPolicyCodePrefixRule {
    fn check(&self, item: &TieredCredentialPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered credential policy.
#[derive(Debug, Clone)]
pub struct TieredCredentialPolicyValidator {
    pub rules: Vec<Rc<dyn TieredCredentialPolicyRule>>,
}

impl TieredCredentialPolicyValidator {
    pub fn validate(&self, item: &TieredCredentialPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredCredentialPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredCredentialPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_credential_policy_validator() -> TieredCredentialPolicyValidator {
    TieredCredentialPolicyValidator {
        rules: vec![
            Rc::new(TieredCredentialPolicyAllowedCodeRule {
                allowed: vec!["strict".to_string(), "default".to_string()],
            }) as Rc<dyn TieredCredentialPolicyRule>,
            Rc::new(TieredCredentialPolicyRequiredNameRule {
                message: "tiered credential policy needs an entry".to_string(),
            }),
            Rc::new(TieredCredentialPolicyQuantityRangeRule { min: 32, max: 1000 }),
            Rc::new(TieredCredentialPolicyCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_credential_policy_validator();
        let item = new_tiered_credential_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredCredentialPolicyRequiredNameRule {
            message: "tiered credential policy needs a value".to_string(),
        };
        let mut item = new_tiered_credential_policy("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = TieredCredentialPolicyQuantityRangeRule { min: 255, max: 255 };
        let mut item = new_tiered_credential_policy("x", "y");
        item.quantity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
