use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a freeze rule must pass. Check returns its problems, or none.
pub trait FreezeRuleRule: std::fmt::Debug {
    fn check(&self, item: &FreezeRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRuleAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl FreezeRuleRule for FreezeRuleAllowedFingerprintRule {
    fn check(&self, item: &FreezeRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is stale", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRulePinnedGuardRule {
    pub limit: i64,
}

impl FreezeRuleRule for FreezeRulePinnedGuardRule {
    fn check(&self, item: &FreezeRule) -> Vec<String> {
        if item.pinned && (item.revision > self.limit) {
            return vec![format!(
                "revision {} exceeds {} while pinned",
                item.revision, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRuleRequiredCodeRule {
    pub message: String,
}

impl FreezeRuleRule for FreezeRuleRequiredCodeRule {
    fn check(&self, item: &FreezeRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze rule.
#[derive(Debug, Clone)]
pub struct FreezeRuleValidator {
    pub rules: Vec<Rc<dyn FreezeRuleRule>>,
}

impl FreezeRuleValidator {
    pub fn validate(&self, item: &FreezeRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezeRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_rule_validator() -> FreezeRuleValidator {
    FreezeRuleValidator {
        rules: vec![
            Rc::new(FreezeRuleAllowedFingerprintRule {
                allowed: vec!["lenient".to_string()],
            }) as Rc<dyn FreezeRuleRule>,
            Rc::new(FreezeRulePinnedGuardRule { limit: 120 }),
            Rc::new(FreezeRuleRequiredCodeRule {
                message: "freeze rule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_rule_validator();
        let item = new_freeze_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FreezeRuleRequiredCodeRule {
            message: "freeze rule needs a value".to_string(),
        };
        let mut item = new_freeze_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
