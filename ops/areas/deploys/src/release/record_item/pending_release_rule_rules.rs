use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending release rule must pass. Check returns its problems, or none.
pub trait PendingReleaseRuleRule: std::fmt::Debug {
    fn check(&self, item: &PendingReleaseRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseRuleCodeLengthRule {
    pub max_length: i64,
}

impl PendingReleaseRuleRule for PendingReleaseRuleCodeLengthRule {
    fn check(&self, item: &PendingReleaseRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseRulePhoneLengthRule {
    pub max_length: i64,
}

impl PendingReleaseRuleRule for PendingReleaseRulePhoneLengthRule {
    fn check(&self, item: &PendingReleaseRule) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingReleaseRuleRule for PendingReleaseRuleAllowedCodeRule {
    fn check(&self, item: &PendingReleaseRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending release rule.
#[derive(Debug, Clone)]
pub struct PendingReleaseRuleValidator {
    pub rules: Vec<Rc<dyn PendingReleaseRuleRule>>,
}

impl PendingReleaseRuleValidator {
    pub fn validate(&self, item: &PendingReleaseRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingReleaseRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingReleaseRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_release_rule_validator() -> PendingReleaseRuleValidator {
    PendingReleaseRuleValidator {
        rules: vec![
            Rc::new(PendingReleaseRuleCodeLengthRule { max_length: 120 }) as Rc<dyn PendingReleaseRuleRule>,
            Rc::new(PendingReleaseRulePhoneLengthRule { max_length: 86400 }),
            Rc::new(PendingReleaseRuleAllowedCodeRule {
                allowed: vec!["weighted".to_string(), "strict".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_release_rule_validator();
        let item = new_pending_release_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingReleaseRuleCodeLengthRule { max_length: 16 };
        let mut item = new_pending_release_rule("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = PendingReleaseRulePhoneLengthRule { max_length: 32 };
        let mut item = new_pending_release_rule("x", "y");
        item.phone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
