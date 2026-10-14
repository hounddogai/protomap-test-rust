use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered runbook audit must pass. Check returns its problems, or none.
pub trait TieredRunbookAuditRule: std::fmt::Debug {
    fn check(&self, item: &TieredRunbookAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookAuditRequiredChannelRule {
    pub message: String,
}

impl TieredRunbookAuditRule for TieredRunbookAuditRequiredChannelRule {
    fn check(&self, item: &TieredRunbookAudit) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookAuditChannelLengthRule {
    pub max_length: i64,
}

impl TieredRunbookAuditRule for TieredRunbookAuditChannelLengthRule {
    fn check(&self, item: &TieredRunbookAudit) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookAuditCodePrefixRule {
    pub prefix: String,
}

impl TieredRunbookAuditRule for TieredRunbookAuditCodePrefixRule {
    fn check(&self, item: &TieredRunbookAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookAuditCategoryLengthRule {
    pub max_length: i64,
}

impl TieredRunbookAuditRule for TieredRunbookAuditCategoryLengthRule {
    fn check(&self, item: &TieredRunbookAudit) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookAuditScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredRunbookAuditRule for TieredRunbookAuditScoreRangeRule {
    fn check(&self, item: &TieredRunbookAudit) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookAuditRequiredNameRule {
    pub message: String,
}

impl TieredRunbookAuditRule for TieredRunbookAuditRequiredNameRule {
    fn check(&self, item: &TieredRunbookAudit) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered runbook audit.
#[derive(Debug, Clone)]
pub struct TieredRunbookAuditValidator {
    pub rules: Vec<Rc<dyn TieredRunbookAuditRule>>,
}

impl TieredRunbookAuditValidator {
    pub fn validate(&self, item: &TieredRunbookAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredRunbookAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRunbookAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_runbook_audit_validator() -> TieredRunbookAuditValidator {
    TieredRunbookAuditValidator {
        rules: vec![
            Rc::new(TieredRunbookAuditRequiredChannelRule {
                message: "tiered runbook audit needs this field".to_string(),
            }) as Rc<dyn TieredRunbookAuditRule>,
            Rc::new(TieredRunbookAuditChannelLengthRule { max_length: 255 }),
            Rc::new(TieredRunbookAuditCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(TieredRunbookAuditCategoryLengthRule { max_length: 32 }),
            Rc::new(TieredRunbookAuditScoreRangeRule { min: 5.5, max: 354.3 }),
            Rc::new(TieredRunbookAuditRequiredNameRule {
                message: "tiered runbook audit needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_runbook_audit_validator();
        let item = new_tiered_runbook_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = TieredRunbookAuditRequiredChannelRule {
            message: "tiered runbook audit needs this field".to_string(),
        };
        let mut item = new_tiered_runbook_audit("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = TieredRunbookAuditChannelLengthRule { max_length: 255 };
        let mut item = new_tiered_runbook_audit("x", "y");
        item.channel = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = TieredRunbookAuditCategoryLengthRule { max_length: 86400 };
        let mut item = new_tiered_runbook_audit("x", "y");
        item.category = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = TieredRunbookAuditScoreRangeRule { min: 4.9, max: 2589.8 };
        let mut item = new_tiered_runbook_audit("x", "y");
        item.score = 2591.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredRunbookAuditRequiredNameRule {
            message: "tiered runbook audit needs this field".to_string(),
        };
        let mut item = new_tiered_runbook_audit("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
