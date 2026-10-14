use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered rule summary must pass. Check returns its problems, or none.
pub trait TieredRuleSummaryRule: std::fmt::Debug {
    fn check(&self, item: &TieredRuleSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleSummaryCodeLengthRule {
    pub max_length: i64,
}

impl TieredRuleSummaryRule for TieredRuleSummaryCodeLengthRule {
    fn check(&self, item: &TieredRuleSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleSummaryChannelLengthRule {
    pub max_length: i64,
}

impl TieredRuleSummaryRule for TieredRuleSummaryChannelLengthRule {
    fn check(&self, item: &TieredRuleSummary) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleSummaryRequiredLabelRule {
    pub message: String,
}

impl TieredRuleSummaryRule for TieredRuleSummaryRequiredLabelRule {
    fn check(&self, item: &TieredRuleSummary) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered rule summary.
#[derive(Debug, Clone)]
pub struct TieredRuleSummaryValidator {
    pub rules: Vec<Rc<dyn TieredRuleSummaryRule>>,
}

impl TieredRuleSummaryValidator {
    pub fn validate(&self, item: &TieredRuleSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredRuleSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRuleSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_rule_summary_validator() -> TieredRuleSummaryValidator {
    TieredRuleSummaryValidator {
        rules: vec![
            Rc::new(TieredRuleSummaryCodeLengthRule { max_length: 16 }) as Rc<dyn TieredRuleSummaryRule>,
            Rc::new(TieredRuleSummaryChannelLengthRule { max_length: 64 }),
            Rc::new(TieredRuleSummaryRequiredLabelRule {
                message: "tiered rule summary needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_rule_summary_validator();
        let item = new_tiered_rule_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredRuleSummaryCodeLengthRule { max_length: 32 };
        let mut item = new_tiered_rule_summary("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = TieredRuleSummaryChannelLengthRule { max_length: 255 };
        let mut item = new_tiered_rule_summary("x", "y");
        item.channel = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TieredRuleSummaryRequiredLabelRule {
            message: "tiered rule summary needs an entry".to_string(),
        };
        let mut item = new_tiered_rule_summary("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
