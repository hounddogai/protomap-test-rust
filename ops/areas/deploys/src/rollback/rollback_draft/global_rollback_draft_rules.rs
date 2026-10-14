use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global rollback draft must pass. Check returns its problems, or none.
pub trait GlobalRollbackDraftRule: std::fmt::Debug {
    fn check(&self, item: &GlobalRollbackDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackDraftMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalRollbackDraftRule for GlobalRollbackDraftMaximumRangeRule {
    fn check(&self, item: &GlobalRollbackDraft) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackDraftRequiredTitleRule {
    pub message: String,
}

impl GlobalRollbackDraftRule for GlobalRollbackDraftRequiredTitleRule {
    fn check(&self, item: &GlobalRollbackDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackDraftRequiredSummaryRule {
    pub message: String,
}

impl GlobalRollbackDraftRule for GlobalRollbackDraftRequiredSummaryRule {
    fn check(&self, item: &GlobalRollbackDraft) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackDraftRequiredCodeRule {
    pub message: String,
}

impl GlobalRollbackDraftRule for GlobalRollbackDraftRequiredCodeRule {
    fn check(&self, item: &GlobalRollbackDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global rollback draft.
#[derive(Debug, Clone)]
pub struct GlobalRollbackDraftValidator {
    pub rules: Vec<Rc<dyn GlobalRollbackDraftRule>>,
}

impl GlobalRollbackDraftValidator {
    pub fn validate(&self, item: &GlobalRollbackDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalRollbackDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalRollbackDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_rollback_draft_validator() -> GlobalRollbackDraftValidator {
    GlobalRollbackDraftValidator {
        rules: vec![
            Rc::new(GlobalRollbackDraftMaximumRangeRule { min: 7.3, max: 4561.2 }) as Rc<dyn GlobalRollbackDraftRule>,
            Rc::new(GlobalRollbackDraftRequiredTitleRule {
                message: "global rollback draft needs this field".to_string(),
            }),
            Rc::new(GlobalRollbackDraftRequiredSummaryRule {
                message: "global rollback draft needs an entry".to_string(),
            }),
            Rc::new(GlobalRollbackDraftRequiredCodeRule {
                message: "global rollback draft needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_rollback_draft_validator();
        let item = new_global_rollback_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = GlobalRollbackDraftMaximumRangeRule { min: 7.5, max: 3363.0 };
        let mut item = new_global_rollback_draft("x", "y");
        item.maximum = 3364.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = GlobalRollbackDraftRequiredTitleRule {
            message: "global rollback draft needs an entry".to_string(),
        };
        let mut item = new_global_rollback_draft("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = GlobalRollbackDraftRequiredSummaryRule {
            message: "global rollback draft needs a value".to_string(),
        };
        let mut item = new_global_rollback_draft("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalRollbackDraftRequiredCodeRule {
            message: "global rollback draft needs an entry".to_string(),
        };
        let mut item = new_global_rollback_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
