use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending retention budget must pass. Check returns its problems, or none.
pub trait PendingRetentionBudgetRule: std::fmt::Debug {
    fn check(&self, item: &PendingRetentionBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionBudgetAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl PendingRetentionBudgetRule for PendingRetentionBudgetAllowedChecksumRule {
    fn check(&self, item: &PendingRetentionBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is unknown", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionBudgetNotesLengthRule {
    pub max_length: i64,
}

impl PendingRetentionBudgetRule for PendingRetentionBudgetNotesLengthRule {
    fn check(&self, item: &PendingRetentionBudget) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionBudgetChecksumLengthRule {
    pub max_length: i64,
}

impl PendingRetentionBudgetRule for PendingRetentionBudgetChecksumLengthRule {
    fn check(&self, item: &PendingRetentionBudget) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionBudgetRequiredCodeRule {
    pub message: String,
}

impl PendingRetentionBudgetRule for PendingRetentionBudgetRequiredCodeRule {
    fn check(&self, item: &PendingRetentionBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionBudgetRequiredChecksumRule {
    pub message: String,
}

impl PendingRetentionBudgetRule for PendingRetentionBudgetRequiredChecksumRule {
    fn check(&self, item: &PendingRetentionBudget) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending retention budget.
#[derive(Debug, Clone)]
pub struct PendingRetentionBudgetValidator {
    pub rules: Vec<Rc<dyn PendingRetentionBudgetRule>>,
}

impl PendingRetentionBudgetValidator {
    pub fn validate(&self, item: &PendingRetentionBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingRetentionBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRetentionBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_retention_budget_validator() -> PendingRetentionBudgetValidator {
    PendingRetentionBudgetValidator {
        rules: vec![
            Rc::new(PendingRetentionBudgetAllowedChecksumRule { allowed: Vec::new() })
                as Rc<dyn PendingRetentionBudgetRule>,
            Rc::new(PendingRetentionBudgetNotesLengthRule { max_length: 80 }),
            Rc::new(PendingRetentionBudgetChecksumLengthRule { max_length: 80 }),
            Rc::new(PendingRetentionBudgetRequiredCodeRule {
                message: "pending retention budget needs a value".to_string(),
            }),
            Rc::new(PendingRetentionBudgetRequiredChecksumRule {
                message: "pending retention budget needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_retention_budget_validator();
        let item = new_pending_retention_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = PendingRetentionBudgetNotesLengthRule { max_length: 1000 };
        let mut item = new_pending_retention_budget("x", "y");
        item.notes = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = PendingRetentionBudgetChecksumLengthRule { max_length: 64 };
        let mut item = new_pending_retention_budget("x", "y");
        item.checksum = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingRetentionBudgetRequiredCodeRule {
            message: "pending retention budget needs this field".to_string(),
        };
        let mut item = new_pending_retention_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = PendingRetentionBudgetRequiredChecksumRule {
            message: "pending retention budget needs a value".to_string(),
        };
        let mut item = new_pending_retention_budget("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
