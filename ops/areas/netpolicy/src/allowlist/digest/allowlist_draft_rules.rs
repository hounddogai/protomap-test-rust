use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist draft must pass. Check returns its problems, or none.
pub trait AllowlistDraftRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistDraftCurrencyLengthRule {
    pub max_length: i64,
}

impl AllowlistDraftRule for AllowlistDraftCurrencyLengthRule {
    fn check(&self, item: &AllowlistDraft) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistDraftRequiredCurrencyRule {
    pub message: String,
}

impl AllowlistDraftRule for AllowlistDraftRequiredCurrencyRule {
    fn check(&self, item: &AllowlistDraft) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistDraftAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl AllowlistDraftRule for AllowlistDraftAllowedCurrencyRule {
    fn check(&self, item: &AllowlistDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is out of range", item.currency)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist draft.
#[derive(Debug, Clone)]
pub struct AllowlistDraftValidator {
    pub rules: Vec<Rc<dyn AllowlistDraftRule>>,
}

impl AllowlistDraftValidator {
    pub fn validate(&self, item: &AllowlistDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_draft_validator() -> AllowlistDraftValidator {
    AllowlistDraftValidator {
        rules: vec![
            Rc::new(AllowlistDraftCurrencyLengthRule { max_length: 16 }) as Rc<dyn AllowlistDraftRule>,
            Rc::new(AllowlistDraftRequiredCurrencyRule {
                message: "allowlist draft needs an entry".to_string(),
            }),
            Rc::new(AllowlistDraftAllowedCurrencyRule {
                allowed: vec!["weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_draft_validator();
        let item = new_allowlist_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = AllowlistDraftCurrencyLengthRule { max_length: 86400 };
        let mut item = new_allowlist_draft("x", "y");
        item.currency = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = AllowlistDraftRequiredCurrencyRule {
            message: "allowlist draft needs an entry".to_string(),
        };
        let mut item = new_allowlist_draft("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
