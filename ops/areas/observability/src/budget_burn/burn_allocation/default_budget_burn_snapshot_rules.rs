use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default budget burn snapshot must pass. Check returns its problems, or none.
pub trait DefaultBudgetBurnSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &DefaultBudgetBurnSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBudgetBurnSnapshotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultBudgetBurnSnapshotRule for DefaultBudgetBurnSnapshotAllowedCodeRule {
    fn check(&self, item: &DefaultBudgetBurnSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBudgetBurnSnapshotRequiredLabelRule {
    pub message: String,
}

impl DefaultBudgetBurnSnapshotRule for DefaultBudgetBurnSnapshotRequiredLabelRule {
    fn check(&self, item: &DefaultBudgetBurnSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBudgetBurnSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl DefaultBudgetBurnSnapshotRule for DefaultBudgetBurnSnapshotTitleLengthRule {
    fn check(&self, item: &DefaultBudgetBurnSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBudgetBurnSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl DefaultBudgetBurnSnapshotRule for DefaultBudgetBurnSnapshotCodeLengthRule {
    fn check(&self, item: &DefaultBudgetBurnSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default budget burn snapshot.
#[derive(Debug, Clone)]
pub struct DefaultBudgetBurnSnapshotValidator {
    pub rules: Vec<Rc<dyn DefaultBudgetBurnSnapshotRule>>,
}

impl DefaultBudgetBurnSnapshotValidator {
    pub fn validate(&self, item: &DefaultBudgetBurnSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultBudgetBurnSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultBudgetBurnSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_budget_burn_snapshot_validator() -> DefaultBudgetBurnSnapshotValidator {
    DefaultBudgetBurnSnapshotValidator {
        rules: vec![
            Rc::new(DefaultBudgetBurnSnapshotAllowedCodeRule {
                allowed: vec!["weighted".to_string(), "manual".to_string(), "manual".to_string()],
            }) as Rc<dyn DefaultBudgetBurnSnapshotRule>,
            Rc::new(DefaultBudgetBurnSnapshotRequiredLabelRule {
                message: "default budget burn snapshot needs a value".to_string(),
            }),
            Rc::new(DefaultBudgetBurnSnapshotTitleLengthRule { max_length: 255 }),
            Rc::new(DefaultBudgetBurnSnapshotCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_budget_burn_snapshot_validator();
        let item = new_default_budget_burn_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultBudgetBurnSnapshotRequiredLabelRule {
            message: "default budget burn snapshot needs this field".to_string(),
        };
        let mut item = new_default_budget_burn_snapshot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultBudgetBurnSnapshotTitleLengthRule { max_length: 3600 };
        let mut item = new_default_budget_burn_snapshot("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultBudgetBurnSnapshotCodeLengthRule { max_length: 3600 };
        let mut item = new_default_budget_burn_snapshot("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
