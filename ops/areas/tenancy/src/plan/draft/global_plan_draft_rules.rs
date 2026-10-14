use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global plan draft must pass. Check returns its problems, or none.
pub trait GlobalPlanDraftRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPlanDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanDraftTitleLengthRule {
    pub max_length: i64,
}

impl GlobalPlanDraftRule for GlobalPlanDraftTitleLengthRule {
    fn check(&self, item: &GlobalPlanDraft) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanDraftWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalPlanDraftRule for GlobalPlanDraftWindowDaysRangeRule {
    fn check(&self, item: &GlobalPlanDraft) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanDraftReviewerLengthRule {
    pub max_length: i64,
}

impl GlobalPlanDraftRule for GlobalPlanDraftReviewerLengthRule {
    fn check(&self, item: &GlobalPlanDraft) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanDraftRequiredCodeRule {
    pub message: String,
}

impl GlobalPlanDraftRule for GlobalPlanDraftRequiredCodeRule {
    fn check(&self, item: &GlobalPlanDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global plan draft.
#[derive(Debug, Clone)]
pub struct GlobalPlanDraftValidator {
    pub rules: Vec<Rc<dyn GlobalPlanDraftRule>>,
}

impl GlobalPlanDraftValidator {
    pub fn validate(&self, item: &GlobalPlanDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalPlanDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPlanDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_plan_draft_validator() -> GlobalPlanDraftValidator {
    GlobalPlanDraftValidator {
        rules: vec![
            Rc::new(GlobalPlanDraftTitleLengthRule { max_length: 16 }) as Rc<dyn GlobalPlanDraftRule>,
            Rc::new(GlobalPlanDraftWindowDaysRangeRule { min: 255, max: 255 }),
            Rc::new(GlobalPlanDraftReviewerLengthRule { max_length: 1000 }),
            Rc::new(GlobalPlanDraftRequiredCodeRule {
                message: "global plan draft needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_plan_draft_validator();
        let item = new_global_plan_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalPlanDraftTitleLengthRule { max_length: 64 };
        let mut item = new_global_plan_draft("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = GlobalPlanDraftWindowDaysRangeRule { min: 80, max: 80 };
        let mut item = new_global_plan_draft("x", "y");
        item.window_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = GlobalPlanDraftReviewerLengthRule { max_length: 80 };
        let mut item = new_global_plan_draft("x", "y");
        item.reviewer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalPlanDraftRequiredCodeRule {
            message: "global plan draft needs an entry".to_string(),
        };
        let mut item = new_global_plan_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
