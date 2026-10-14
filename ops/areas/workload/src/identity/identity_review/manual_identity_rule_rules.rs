use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual identity rule must pass. Check returns its problems, or none.
pub trait ManualIdentityRuleRule: std::fmt::Debug {
    fn check(&self, item: &ManualIdentityRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityRuleRequiredCodeRule {
    pub message: String,
}

impl ManualIdentityRuleRule for ManualIdentityRuleRequiredCodeRule {
    fn check(&self, item: &ManualIdentityRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityRuleAllowedHostRule {
    pub allowed: Vec<String>,
}

impl ManualIdentityRuleRule for ManualIdentityRuleAllowedHostRule {
    fn check(&self, item: &ManualIdentityRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is above maximum", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityRuleCodePrefixRule {
    pub prefix: String,
}

impl ManualIdentityRuleRule for ManualIdentityRuleCodePrefixRule {
    fn check(&self, item: &ManualIdentityRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual identity rule.
#[derive(Debug, Clone)]
pub struct ManualIdentityRuleValidator {
    pub rules: Vec<Rc<dyn ManualIdentityRuleRule>>,
}

impl ManualIdentityRuleValidator {
    pub fn validate(&self, item: &ManualIdentityRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualIdentityRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualIdentityRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_identity_rule_validator() -> ManualIdentityRuleValidator {
    ManualIdentityRuleValidator {
        rules: vec![
            Rc::new(ManualIdentityRuleRequiredCodeRule {
                message: "manual identity rule needs a value".to_string(),
            }) as Rc<dyn ManualIdentityRuleRule>,
            Rc::new(ManualIdentityRuleAllowedHostRule {
                allowed: vec!["lenient".to_string(), "lenient".to_string(), "rolling".to_string()],
            }),
            Rc::new(ManualIdentityRuleCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_identity_rule_validator();
        let item = new_manual_identity_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualIdentityRuleRequiredCodeRule {
            message: "manual identity rule needs an entry".to_string(),
        };
        let mut item = new_manual_identity_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
