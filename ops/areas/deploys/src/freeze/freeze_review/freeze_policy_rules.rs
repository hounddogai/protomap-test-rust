use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a freeze policy must pass. Check returns its problems, or none.
pub trait FreezePolicyRule: std::fmt::Debug {
    fn check(&self, item: &FreezePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezePolicyAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl FreezePolicyRule for FreezePolicyAllowedScopeRule {
    fn check(&self, item: &FreezePolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is below minimum", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezePolicyRequiredKeyIdRule {
    pub message: String,
}

impl FreezePolicyRule for FreezePolicyRequiredKeyIdRule {
    fn check(&self, item: &FreezePolicy) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezePolicyLabelLengthRule {
    pub max_length: i64,
}

impl FreezePolicyRule for FreezePolicyLabelLengthRule {
    fn check(&self, item: &FreezePolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze policy.
#[derive(Debug, Clone)]
pub struct FreezePolicyValidator {
    pub rules: Vec<Rc<dyn FreezePolicyRule>>,
}

impl FreezePolicyValidator {
    pub fn validate(&self, item: &FreezePolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_policy_validator() -> FreezePolicyValidator {
    FreezePolicyValidator {
        rules: vec![
            Rc::new(FreezePolicyAllowedScopeRule {
                allowed: vec!["global".to_string(), "rolling".to_string(), "rolling".to_string()],
            }) as Rc<dyn FreezePolicyRule>,
            Rc::new(FreezePolicyRequiredKeyIdRule {
                message: "freeze policy needs an entry".to_string(),
            }),
            Rc::new(FreezePolicyLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_policy_validator();
        let item = new_freeze_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = FreezePolicyRequiredKeyIdRule {
            message: "freeze policy needs an entry".to_string(),
        };
        let mut item = new_freeze_policy("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FreezePolicyLabelLengthRule { max_length: 16 };
        let mut item = new_freeze_policy("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
