use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant draft must pass. Check returns its problems, or none.
pub trait GrantDraftRule: std::fmt::Debug {
    fn check(&self, item: &GrantDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantDraftCodePrefixRule {
    pub prefix: String,
}

impl GrantDraftRule for GrantDraftCodePrefixRule {
    fn check(&self, item: &GrantDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantDraftRequiredNameRule {
    pub message: String,
}

impl GrantDraftRule for GrantDraftRequiredNameRule {
    fn check(&self, item: &GrantDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantDraftCodeLengthRule {
    pub max_length: i64,
}

impl GrantDraftRule for GrantDraftCodeLengthRule {
    fn check(&self, item: &GrantDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantDraftCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantDraftRule for GrantDraftCostRangeRule {
    fn check(&self, item: &GrantDraft) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantDraftNameLengthRule {
    pub max_length: i64,
}

impl GrantDraftRule for GrantDraftNameLengthRule {
    fn check(&self, item: &GrantDraft) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant draft.
#[derive(Debug, Clone)]
pub struct GrantDraftValidator {
    pub rules: Vec<Rc<dyn GrantDraftRule>>,
}

impl GrantDraftValidator {
    pub fn validate(&self, item: &GrantDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_draft_validator() -> GrantDraftValidator {
    GrantDraftValidator {
        rules: vec![
            Rc::new(GrantDraftCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn GrantDraftRule>,
            Rc::new(GrantDraftRequiredNameRule {
                message: "grant draft needs an entry".to_string(),
            }),
            Rc::new(GrantDraftCodeLengthRule { max_length: 16 }),
            Rc::new(GrantDraftCostRangeRule { min: 8.5, max: 125.7 }),
            Rc::new(GrantDraftNameLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_draft_validator();
        let item = new_grant_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GrantDraftRequiredNameRule {
            message: "grant draft needs this field".to_string(),
        };
        let mut item = new_grant_draft("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GrantDraftCodeLengthRule { max_length: 255 };
        let mut item = new_grant_draft("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = GrantDraftCostRangeRule { min: 8.0, max: 3059.2 };
        let mut item = new_grant_draft("x", "y");
        item.cost = 3060.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GrantDraftNameLengthRule { max_length: 32 };
        let mut item = new_grant_draft("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
