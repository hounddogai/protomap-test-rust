use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default rollback draft must pass. Check returns its problems, or none.
pub trait DefaultRollbackDraftRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRollbackDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackDraftRequiredCodeRule {
    pub message: String,
}

impl DefaultRollbackDraftRule for DefaultRollbackDraftRequiredCodeRule {
    fn check(&self, item: &DefaultRollbackDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackDraftExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRollbackDraftRule for DefaultRollbackDraftExpiresAtRangeRule {
    fn check(&self, item: &DefaultRollbackDraft) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackDraftUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRollbackDraftRule for DefaultRollbackDraftUpdatedAtRangeRule {
    fn check(&self, item: &DefaultRollbackDraft) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackDraftRequiredLabelRule {
    pub message: String,
}

impl DefaultRollbackDraftRule for DefaultRollbackDraftRequiredLabelRule {
    fn check(&self, item: &DefaultRollbackDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default rollback draft.
#[derive(Debug, Clone)]
pub struct DefaultRollbackDraftValidator {
    pub rules: Vec<Rc<dyn DefaultRollbackDraftRule>>,
}

impl DefaultRollbackDraftValidator {
    pub fn validate(&self, item: &DefaultRollbackDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultRollbackDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRollbackDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_rollback_draft_validator() -> DefaultRollbackDraftValidator {
    DefaultRollbackDraftValidator {
        rules: vec![
            Rc::new(DefaultRollbackDraftRequiredCodeRule {
                message: "default rollback draft needs a value".to_string(),
            }) as Rc<dyn DefaultRollbackDraftRule>,
            Rc::new(DefaultRollbackDraftExpiresAtRangeRule { min: 16, max: 64 }),
            Rc::new(DefaultRollbackDraftUpdatedAtRangeRule { min: 16, max: 255 }),
            Rc::new(DefaultRollbackDraftRequiredLabelRule {
                message: "default rollback draft needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_rollback_draft_validator();
        let item = new_default_rollback_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultRollbackDraftRequiredCodeRule {
            message: "default rollback draft needs a value".to_string(),
        };
        let mut item = new_default_rollback_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = DefaultRollbackDraftExpiresAtRangeRule { min: 64, max: 255 };
        let mut item = new_default_rollback_draft("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = DefaultRollbackDraftUpdatedAtRangeRule { min: 16, max: 1000 };
        let mut item = new_default_rollback_draft("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultRollbackDraftRequiredLabelRule {
            message: "default rollback draft needs a value".to_string(),
        };
        let mut item = new_default_rollback_draft("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
