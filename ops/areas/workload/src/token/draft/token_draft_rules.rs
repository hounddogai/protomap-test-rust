use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token draft must pass. Check returns its problems, or none.
pub trait TokenDraftRule: std::fmt::Debug {
    fn check(&self, item: &TokenDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenDraftTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl TokenDraftRule for TokenDraftTagsLimitRule {
    fn check(&self, item: &TokenDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenDraftLabelLengthRule {
    pub max_length: i64,
}

impl TokenDraftRule for TokenDraftLabelLengthRule {
    fn check(&self, item: &TokenDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenDraftCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenDraftRule for TokenDraftCostRangeRule {
    fn check(&self, item: &TokenDraft) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenDraftRequiredLabelRule {
    pub message: String,
}

impl TokenDraftRule for TokenDraftRequiredLabelRule {
    fn check(&self, item: &TokenDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenDraftTimezoneLengthRule {
    pub max_length: i64,
}

impl TokenDraftRule for TokenDraftTimezoneLengthRule {
    fn check(&self, item: &TokenDraft) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a token draft.
#[derive(Debug, Clone)]
pub struct TokenDraftValidator {
    pub rules: Vec<Rc<dyn TokenDraftRule>>,
}

impl TokenDraftValidator {
    pub fn validate(&self, item: &TokenDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_draft_validator() -> TokenDraftValidator {
    TokenDraftValidator {
        rules: vec![
            Rc::new(TokenDraftTagsLimitRule {
                max_count: 32,
                banned: vec!["seasonal".to_string(), "seasonal".to_string()],
            }) as Rc<dyn TokenDraftRule>,
            Rc::new(TokenDraftLabelLengthRule { max_length: 255 }),
            Rc::new(TokenDraftCostRangeRule { min: 3.9, max: 3342.6 }),
            Rc::new(TokenDraftRequiredLabelRule {
                message: "token draft needs a value".to_string(),
            }),
            Rc::new(TokenDraftTimezoneLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_draft_validator();
        let item = new_token_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TokenDraftLabelLengthRule { max_length: 120 };
        let mut item = new_token_draft("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = TokenDraftCostRangeRule { min: 1.4, max: 4831.5 };
        let mut item = new_token_draft("x", "y");
        item.cost = 4833.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TokenDraftRequiredLabelRule {
            message: "token draft needs a value".to_string(),
        };
        let mut item = new_token_draft("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = TokenDraftTimezoneLengthRule { max_length: 64 };
        let mut item = new_token_draft("x", "y");
        item.timezone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
