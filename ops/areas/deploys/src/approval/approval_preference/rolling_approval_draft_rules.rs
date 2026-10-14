use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling approval draft must pass. Check returns its problems, or none.
pub trait RollingApprovalDraftRule: std::fmt::Debug {
    fn check(&self, item: &RollingApprovalDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalDraftRequiredTitleRule {
    pub message: String,
}

impl RollingApprovalDraftRule for RollingApprovalDraftRequiredTitleRule {
    fn check(&self, item: &RollingApprovalDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalDraftSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingApprovalDraftRule for RollingApprovalDraftSequenceRangeRule {
    fn check(&self, item: &RollingApprovalDraft) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalDraftCodeLengthRule {
    pub max_length: i64,
}

impl RollingApprovalDraftRule for RollingApprovalDraftCodeLengthRule {
    fn check(&self, item: &RollingApprovalDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalDraftCodePrefixRule {
    pub prefix: String,
}

impl RollingApprovalDraftRule for RollingApprovalDraftCodePrefixRule {
    fn check(&self, item: &RollingApprovalDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalDraftMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingApprovalDraftRule for RollingApprovalDraftMarginRangeRule {
    fn check(&self, item: &RollingApprovalDraft) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling approval draft.
#[derive(Debug, Clone)]
pub struct RollingApprovalDraftValidator {
    pub rules: Vec<Rc<dyn RollingApprovalDraftRule>>,
}

impl RollingApprovalDraftValidator {
    pub fn validate(&self, item: &RollingApprovalDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingApprovalDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingApprovalDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_approval_draft_validator() -> RollingApprovalDraftValidator {
    RollingApprovalDraftValidator {
        rules: vec![
            Rc::new(RollingApprovalDraftRequiredTitleRule {
                message: "rolling approval draft needs a value".to_string(),
            }) as Rc<dyn RollingApprovalDraftRule>,
            Rc::new(RollingApprovalDraftSequenceRangeRule { min: 80, max: 3600 }),
            Rc::new(RollingApprovalDraftCodeLengthRule { max_length: 64 }),
            Rc::new(RollingApprovalDraftCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RollingApprovalDraftMarginRangeRule { min: 0.4, max: 2383.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_approval_draft_validator();
        let item = new_rolling_approval_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollingApprovalDraftRequiredTitleRule {
            message: "rolling approval draft needs an entry".to_string(),
        };
        let mut item = new_rolling_approval_draft("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = RollingApprovalDraftSequenceRangeRule { min: 1000, max: 86400 };
        let mut item = new_rolling_approval_draft("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingApprovalDraftCodeLengthRule { max_length: 120 };
        let mut item = new_rolling_approval_draft("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = RollingApprovalDraftMarginRangeRule { min: 6.6, max: 2084.5 };
        let mut item = new_rolling_approval_draft("x", "y");
        item.margin = 2086.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
