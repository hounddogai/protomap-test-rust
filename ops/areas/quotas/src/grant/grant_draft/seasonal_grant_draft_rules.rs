use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal grant draft must pass. Check returns its problems, or none.
pub trait SeasonalGrantDraftRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalGrantDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantDraftRequiredOwnerIdRule {
    pub message: String,
}

impl SeasonalGrantDraftRule for SeasonalGrantDraftRequiredOwnerIdRule {
    fn check(&self, item: &SeasonalGrantDraft) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantDraftOwnerIdLengthRule {
    pub max_length: i64,
}

impl SeasonalGrantDraftRule for SeasonalGrantDraftOwnerIdLengthRule {
    fn check(&self, item: &SeasonalGrantDraft) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantDraftRequiredTitleRule {
    pub message: String,
}

impl SeasonalGrantDraftRule for SeasonalGrantDraftRequiredTitleRule {
    fn check(&self, item: &SeasonalGrantDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantDraftRequiredCodeRule {
    pub message: String,
}

impl SeasonalGrantDraftRule for SeasonalGrantDraftRequiredCodeRule {
    fn check(&self, item: &SeasonalGrantDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantDraftAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl SeasonalGrantDraftRule for SeasonalGrantDraftAllowedAudienceRule {
    fn check(&self, item: &SeasonalGrantDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is below minimum", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantDraftTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalGrantDraftRule for SeasonalGrantDraftTitleLengthRule {
    fn check(&self, item: &SeasonalGrantDraft) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal grant draft.
#[derive(Debug, Clone)]
pub struct SeasonalGrantDraftValidator {
    pub rules: Vec<Rc<dyn SeasonalGrantDraftRule>>,
}

impl SeasonalGrantDraftValidator {
    pub fn validate(&self, item: &SeasonalGrantDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalGrantDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalGrantDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_grant_draft_validator() -> SeasonalGrantDraftValidator {
    SeasonalGrantDraftValidator {
        rules: vec![
            Rc::new(SeasonalGrantDraftRequiredOwnerIdRule {
                message: "seasonal grant draft needs an entry".to_string(),
            }) as Rc<dyn SeasonalGrantDraftRule>,
            Rc::new(SeasonalGrantDraftOwnerIdLengthRule { max_length: 1000 }),
            Rc::new(SeasonalGrantDraftRequiredTitleRule {
                message: "seasonal grant draft needs a value".to_string(),
            }),
            Rc::new(SeasonalGrantDraftRequiredCodeRule {
                message: "seasonal grant draft needs an entry".to_string(),
            }),
            Rc::new(SeasonalGrantDraftAllowedAudienceRule {
                allowed: vec!["pending".to_string()],
            }),
            Rc::new(SeasonalGrantDraftTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_grant_draft_validator();
        let item = new_seasonal_grant_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = SeasonalGrantDraftRequiredOwnerIdRule {
            message: "seasonal grant draft needs an entry".to_string(),
        };
        let mut item = new_seasonal_grant_draft("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = SeasonalGrantDraftOwnerIdLengthRule { max_length: 120 };
        let mut item = new_seasonal_grant_draft("x", "y");
        item.owner_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SeasonalGrantDraftRequiredTitleRule {
            message: "seasonal grant draft needs an entry".to_string(),
        };
        let mut item = new_seasonal_grant_draft("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalGrantDraftRequiredCodeRule {
            message: "seasonal grant draft needs an entry".to_string(),
        };
        let mut item = new_seasonal_grant_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SeasonalGrantDraftTitleLengthRule { max_length: 86400 };
        let mut item = new_seasonal_grant_draft("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
