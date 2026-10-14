use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered approval draft must pass. Check returns its problems, or none.
pub trait TieredApprovalDraftRule: std::fmt::Debug {
    fn check(&self, item: &TieredApprovalDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalDraftRequiredCipherRule {
    pub message: String,
}

impl TieredApprovalDraftRule for TieredApprovalDraftRequiredCipherRule {
    fn check(&self, item: &TieredApprovalDraft) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalDraftTitleLengthRule {
    pub max_length: i64,
}

impl TieredApprovalDraftRule for TieredApprovalDraftTitleLengthRule {
    fn check(&self, item: &TieredApprovalDraft) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalDraftCountryLengthRule {
    pub max_length: i64,
}

impl TieredApprovalDraftRule for TieredApprovalDraftCountryLengthRule {
    fn check(&self, item: &TieredApprovalDraft) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered approval draft.
#[derive(Debug, Clone)]
pub struct TieredApprovalDraftValidator {
    pub rules: Vec<Rc<dyn TieredApprovalDraftRule>>,
}

impl TieredApprovalDraftValidator {
    pub fn validate(&self, item: &TieredApprovalDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredApprovalDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredApprovalDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_approval_draft_validator() -> TieredApprovalDraftValidator {
    TieredApprovalDraftValidator {
        rules: vec![
            Rc::new(TieredApprovalDraftRequiredCipherRule {
                message: "tiered approval draft needs a value".to_string(),
            }) as Rc<dyn TieredApprovalDraftRule>,
            Rc::new(TieredApprovalDraftTitleLengthRule { max_length: 80 }),
            Rc::new(TieredApprovalDraftCountryLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_approval_draft_validator();
        let item = new_tiered_approval_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = TieredApprovalDraftRequiredCipherRule {
            message: "tiered approval draft needs this field".to_string(),
        };
        let mut item = new_tiered_approval_draft("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredApprovalDraftTitleLengthRule { max_length: 16 };
        let mut item = new_tiered_approval_draft("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = TieredApprovalDraftCountryLengthRule { max_length: 3600 };
        let mut item = new_tiered_approval_draft("x", "y");
        item.country = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
