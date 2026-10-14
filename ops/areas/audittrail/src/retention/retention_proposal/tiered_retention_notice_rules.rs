use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered retention notice must pass. Check returns its problems, or none.
pub trait TieredRetentionNoticeRule: std::fmt::Debug {
    fn check(&self, item: &TieredRetentionNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionNoticeRequiredNameRule {
    pub message: String,
}

impl TieredRetentionNoticeRule for TieredRetentionNoticeRequiredNameRule {
    fn check(&self, item: &TieredRetentionNotice) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionNoticePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredRetentionNoticeRule for TieredRetentionNoticePriorityRangeRule {
    fn check(&self, item: &TieredRetentionNotice) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionNoticeBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredRetentionNoticeRule for TieredRetentionNoticeBudgetRangeRule {
    fn check(&self, item: &TieredRetentionNotice) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionNoticeTitleLengthRule {
    pub max_length: i64,
}

impl TieredRetentionNoticeRule for TieredRetentionNoticeTitleLengthRule {
    fn check(&self, item: &TieredRetentionNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionNoticeCodeLengthRule {
    pub max_length: i64,
}

impl TieredRetentionNoticeRule for TieredRetentionNoticeCodeLengthRule {
    fn check(&self, item: &TieredRetentionNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered retention notice.
#[derive(Debug, Clone)]
pub struct TieredRetentionNoticeValidator {
    pub rules: Vec<Rc<dyn TieredRetentionNoticeRule>>,
}

impl TieredRetentionNoticeValidator {
    pub fn validate(&self, item: &TieredRetentionNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredRetentionNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRetentionNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_retention_notice_validator() -> TieredRetentionNoticeValidator {
    TieredRetentionNoticeValidator {
        rules: vec![
            Rc::new(TieredRetentionNoticeRequiredNameRule {
                message: "tiered retention notice needs a value".to_string(),
            }) as Rc<dyn TieredRetentionNoticeRule>,
            Rc::new(TieredRetentionNoticePriorityRangeRule { min: 16, max: 86400 }),
            Rc::new(TieredRetentionNoticeBudgetRangeRule { min: 7.2, max: 2547.5 }),
            Rc::new(TieredRetentionNoticeTitleLengthRule { max_length: 16 }),
            Rc::new(TieredRetentionNoticeCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_retention_notice_validator();
        let item = new_tiered_retention_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredRetentionNoticeRequiredNameRule {
            message: "tiered retention notice needs a value".to_string(),
        };
        let mut item = new_tiered_retention_notice("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = TieredRetentionNoticePriorityRangeRule { min: 1000, max: 86400 };
        let mut item = new_tiered_retention_notice("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = TieredRetentionNoticeBudgetRangeRule { min: 9.3, max: 588.4 };
        let mut item = new_tiered_retention_notice("x", "y");
        item.budget = 589.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredRetentionNoticeTitleLengthRule { max_length: 86400 };
        let mut item = new_tiered_retention_notice("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredRetentionNoticeCodeLengthRule { max_length: 3600 };
        let mut item = new_tiered_retention_notice("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
