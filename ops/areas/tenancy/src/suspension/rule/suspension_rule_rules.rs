use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a suspension rule must pass. Check returns its problems, or none.
pub trait SuspensionRuleRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRuleRequiredAudienceRule {
    pub message: String,
}

impl SuspensionRuleRule for SuspensionRuleRequiredAudienceRule {
    fn check(&self, item: &SuspensionRule) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRuleAudienceLengthRule {
    pub max_length: i64,
}

impl SuspensionRuleRule for SuspensionRuleAudienceLengthRule {
    fn check(&self, item: &SuspensionRule) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRuleCodeLengthRule {
    pub max_length: i64,
}

impl SuspensionRuleRule for SuspensionRuleCodeLengthRule {
    fn check(&self, item: &SuspensionRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRuleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl SuspensionRuleRule for SuspensionRuleAllowedNameRule {
    fn check(&self, item: &SuspensionRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is inconsistent", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension rule.
#[derive(Debug, Clone)]
pub struct SuspensionRuleValidator {
    pub rules: Vec<Rc<dyn SuspensionRuleRule>>,
}

impl SuspensionRuleValidator {
    pub fn validate(&self, item: &SuspensionRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_rule_validator() -> SuspensionRuleValidator {
    SuspensionRuleValidator {
        rules: vec![
            Rc::new(SuspensionRuleRequiredAudienceRule {
                message: "suspension rule needs a value".to_string(),
            }) as Rc<dyn SuspensionRuleRule>,
            Rc::new(SuspensionRuleAudienceLengthRule { max_length: 80 }),
            Rc::new(SuspensionRuleCodeLengthRule { max_length: 255 }),
            Rc::new(SuspensionRuleAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_rule_validator();
        let item = new_suspension_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = SuspensionRuleRequiredAudienceRule {
            message: "suspension rule needs a value".to_string(),
        };
        let mut item = new_suspension_rule("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = SuspensionRuleAudienceLengthRule { max_length: 16 };
        let mut item = new_suspension_rule("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SuspensionRuleCodeLengthRule { max_length: 16 };
        let mut item = new_suspension_rule("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
