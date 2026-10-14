use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict identity target must pass. Check returns its problems, or none.
pub trait StrictIdentityTargetRule: std::fmt::Debug {
    fn check(&self, item: &StrictIdentityTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIdentityTargetRequiredCountryRule {
    pub message: String,
}

impl StrictIdentityTargetRule for StrictIdentityTargetRequiredCountryRule {
    fn check(&self, item: &StrictIdentityTarget) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIdentityTargetCodeLengthRule {
    pub max_length: i64,
}

impl StrictIdentityTargetRule for StrictIdentityTargetCodeLengthRule {
    fn check(&self, item: &StrictIdentityTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIdentityTargetNamespaceLengthRule {
    pub max_length: i64,
}

impl StrictIdentityTargetRule for StrictIdentityTargetNamespaceLengthRule {
    fn check(&self, item: &StrictIdentityTarget) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIdentityTargetCodePrefixRule {
    pub prefix: String,
}

impl StrictIdentityTargetRule for StrictIdentityTargetCodePrefixRule {
    fn check(&self, item: &StrictIdentityTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIdentityTargetCountryLengthRule {
    pub max_length: i64,
}

impl StrictIdentityTargetRule for StrictIdentityTargetCountryLengthRule {
    fn check(&self, item: &StrictIdentityTarget) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict identity target.
#[derive(Debug, Clone)]
pub struct StrictIdentityTargetValidator {
    pub rules: Vec<Rc<dyn StrictIdentityTargetRule>>,
}

impl StrictIdentityTargetValidator {
    pub fn validate(&self, item: &StrictIdentityTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictIdentityTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictIdentityTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_identity_target_validator() -> StrictIdentityTargetValidator {
    StrictIdentityTargetValidator {
        rules: vec![
            Rc::new(StrictIdentityTargetRequiredCountryRule {
                message: "strict identity target needs this field".to_string(),
            }) as Rc<dyn StrictIdentityTargetRule>,
            Rc::new(StrictIdentityTargetCodeLengthRule { max_length: 64 }),
            Rc::new(StrictIdentityTargetNamespaceLengthRule { max_length: 16 }),
            Rc::new(StrictIdentityTargetCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(StrictIdentityTargetCountryLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_identity_target_validator();
        let item = new_strict_identity_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = StrictIdentityTargetRequiredCountryRule {
            message: "strict identity target needs a value".to_string(),
        };
        let mut item = new_strict_identity_target("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictIdentityTargetCodeLengthRule { max_length: 64 };
        let mut item = new_strict_identity_target("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = StrictIdentityTargetNamespaceLengthRule { max_length: 32 };
        let mut item = new_strict_identity_target("x", "y");
        item.namespace = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = StrictIdentityTargetCountryLengthRule { max_length: 1000 };
        let mut item = new_strict_identity_target("x", "y");
        item.country = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
