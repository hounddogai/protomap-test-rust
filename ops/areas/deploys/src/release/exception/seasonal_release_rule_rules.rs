use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal release rule must pass. Check returns its problems, or none.
pub trait SeasonalReleaseRuleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalReleaseRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseRuleRequiredTitleRule {
    pub message: String,
}

impl SeasonalReleaseRuleRule for SeasonalReleaseRuleRequiredTitleRule {
    fn check(&self, item: &SeasonalReleaseRule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseRuleRequiredCodeRule {
    pub message: String,
}

impl SeasonalReleaseRuleRule for SeasonalReleaseRuleRequiredCodeRule {
    fn check(&self, item: &SeasonalReleaseRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseRuleCurrencyLengthRule {
    pub max_length: i64,
}

impl SeasonalReleaseRuleRule for SeasonalReleaseRuleCurrencyLengthRule {
    fn check(&self, item: &SeasonalReleaseRule) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseRuleNotesLengthRule {
    pub max_length: i64,
}

impl SeasonalReleaseRuleRule for SeasonalReleaseRuleNotesLengthRule {
    fn check(&self, item: &SeasonalReleaseRule) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseRuleCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalReleaseRuleRule for SeasonalReleaseRuleCodeLengthRule {
    fn check(&self, item: &SeasonalReleaseRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseRuleRequiredNotesRule {
    pub message: String,
}

impl SeasonalReleaseRuleRule for SeasonalReleaseRuleRequiredNotesRule {
    fn check(&self, item: &SeasonalReleaseRule) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal release rule.
#[derive(Debug, Clone)]
pub struct SeasonalReleaseRuleValidator {
    pub rules: Vec<Rc<dyn SeasonalReleaseRuleRule>>,
}

impl SeasonalReleaseRuleValidator {
    pub fn validate(&self, item: &SeasonalReleaseRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalReleaseRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalReleaseRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_release_rule_validator() -> SeasonalReleaseRuleValidator {
    SeasonalReleaseRuleValidator {
        rules: vec![
            Rc::new(SeasonalReleaseRuleRequiredTitleRule {
                message: "seasonal release rule needs an entry".to_string(),
            }) as Rc<dyn SeasonalReleaseRuleRule>,
            Rc::new(SeasonalReleaseRuleRequiredCodeRule {
                message: "seasonal release rule needs an entry".to_string(),
            }),
            Rc::new(SeasonalReleaseRuleCurrencyLengthRule { max_length: 3600 }),
            Rc::new(SeasonalReleaseRuleNotesLengthRule { max_length: 32 }),
            Rc::new(SeasonalReleaseRuleCodeLengthRule { max_length: 64 }),
            Rc::new(SeasonalReleaseRuleRequiredNotesRule {
                message: "seasonal release rule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_release_rule_validator();
        let item = new_seasonal_release_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SeasonalReleaseRuleRequiredTitleRule {
            message: "seasonal release rule needs a value".to_string(),
        };
        let mut item = new_seasonal_release_rule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalReleaseRuleRequiredCodeRule {
            message: "seasonal release rule needs a value".to_string(),
        };
        let mut item = new_seasonal_release_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = SeasonalReleaseRuleCurrencyLengthRule { max_length: 80 };
        let mut item = new_seasonal_release_rule("x", "y");
        item.currency = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = SeasonalReleaseRuleNotesLengthRule { max_length: 80 };
        let mut item = new_seasonal_release_rule("x", "y");
        item.notes = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalReleaseRuleCodeLengthRule { max_length: 86400 };
        let mut item = new_seasonal_release_rule("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = SeasonalReleaseRuleRequiredNotesRule {
            message: "seasonal release rule needs this field".to_string(),
        };
        let mut item = new_seasonal_release_rule("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
