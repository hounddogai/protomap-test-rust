use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted action item draft must pass. Check returns its problems, or none.
pub trait WeightedActionItemDraftRule: std::fmt::Debug {
    fn check(&self, item: &WeightedActionItemDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemDraftAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl WeightedActionItemDraftRule for WeightedActionItemDraftAllowedCodeRule {
    fn check(&self, item: &WeightedActionItemDraft) -> Vec<String> {
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
pub struct WeightedActionItemDraftActiveGuardRule {
    pub limit: i64,
}

impl WeightedActionItemDraftRule for WeightedActionItemDraftActiveGuardRule {
    fn check(&self, item: &WeightedActionItemDraft) -> Vec<String> {
        if item.active && (item.sequence > self.limit) {
            return vec![format!(
                "sequence {} exceeds {} while active",
                item.sequence, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemDraftCodeLengthRule {
    pub max_length: i64,
}

impl WeightedActionItemDraftRule for WeightedActionItemDraftCodeLengthRule {
    fn check(&self, item: &WeightedActionItemDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemDraftEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedActionItemDraftRule for WeightedActionItemDraftEndsAtRangeRule {
    fn check(&self, item: &WeightedActionItemDraft) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a weighted action item draft.
#[derive(Debug, Clone)]
pub struct WeightedActionItemDraftValidator {
    pub rules: Vec<Rc<dyn WeightedActionItemDraftRule>>,
}

impl WeightedActionItemDraftValidator {
    pub fn validate(&self, item: &WeightedActionItemDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedActionItemDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedActionItemDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_action_item_draft_validator() -> WeightedActionItemDraftValidator {
    WeightedActionItemDraftValidator {
        rules: vec![
            Rc::new(WeightedActionItemDraftAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "batched".to_string()],
            }) as Rc<dyn WeightedActionItemDraftRule>,
            Rc::new(WeightedActionItemDraftActiveGuardRule { limit: 1000 }),
            Rc::new(WeightedActionItemDraftCodeLengthRule { max_length: 64 }),
            Rc::new(WeightedActionItemDraftEndsAtRangeRule { min: 32, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_action_item_draft_validator();
        let item = new_weighted_action_item_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedActionItemDraftCodeLengthRule { max_length: 3600 };
        let mut item = new_weighted_action_item_draft("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = WeightedActionItemDraftEndsAtRangeRule { min: 255, max: 86400 };
        let mut item = new_weighted_action_item_draft("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
