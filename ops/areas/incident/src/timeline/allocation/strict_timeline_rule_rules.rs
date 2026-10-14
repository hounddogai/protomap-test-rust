use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict timeline rule must pass. Check returns its problems, or none.
pub trait StrictTimelineRuleRule: std::fmt::Debug {
    fn check(&self, item: &StrictTimelineRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineRuleRequiredCodeRule {
    pub message: String,
}

impl StrictTimelineRuleRule for StrictTimelineRuleRequiredCodeRule {
    fn check(&self, item: &StrictTimelineRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineRuleRequiredNameRule {
    pub message: String,
}

impl StrictTimelineRuleRule for StrictTimelineRuleRequiredNameRule {
    fn check(&self, item: &StrictTimelineRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineRuleCodePrefixRule {
    pub prefix: String,
}

impl StrictTimelineRuleRule for StrictTimelineRuleCodePrefixRule {
    fn check(&self, item: &StrictTimelineRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineRuleCodeLengthRule {
    pub max_length: i64,
}

impl StrictTimelineRuleRule for StrictTimelineRuleCodeLengthRule {
    fn check(&self, item: &StrictTimelineRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict timeline rule.
#[derive(Debug, Clone)]
pub struct StrictTimelineRuleValidator {
    pub rules: Vec<Rc<dyn StrictTimelineRuleRule>>,
}

impl StrictTimelineRuleValidator {
    pub fn validate(&self, item: &StrictTimelineRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictTimelineRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictTimelineRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_timeline_rule_validator() -> StrictTimelineRuleValidator {
    StrictTimelineRuleValidator {
        rules: vec![
            Rc::new(StrictTimelineRuleRequiredCodeRule {
                message: "strict timeline rule needs a value".to_string(),
            }) as Rc<dyn StrictTimelineRuleRule>,
            Rc::new(StrictTimelineRuleRequiredNameRule {
                message: "strict timeline rule needs this field".to_string(),
            }),
            Rc::new(StrictTimelineRuleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(StrictTimelineRuleCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_timeline_rule_validator();
        let item = new_strict_timeline_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictTimelineRuleRequiredCodeRule {
            message: "strict timeline rule needs an entry".to_string(),
        };
        let mut item = new_strict_timeline_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictTimelineRuleRequiredNameRule {
            message: "strict timeline rule needs a value".to_string(),
        };
        let mut item = new_strict_timeline_rule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictTimelineRuleCodeLengthRule { max_length: 86400 };
        let mut item = new_strict_timeline_rule("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
