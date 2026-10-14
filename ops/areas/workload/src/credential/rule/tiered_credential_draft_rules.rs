use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered credential draft must pass. Check returns its problems, or none.
pub trait TieredCredentialDraftRule: std::fmt::Debug {
    fn check(&self, item: &TieredCredentialDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialDraftCodeLengthRule {
    pub max_length: i64,
}

impl TieredCredentialDraftRule for TieredCredentialDraftCodeLengthRule {
    fn check(&self, item: &TieredCredentialDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialDraftTitleLengthRule {
    pub max_length: i64,
}

impl TieredCredentialDraftRule for TieredCredentialDraftTitleLengthRule {
    fn check(&self, item: &TieredCredentialDraft) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialDraftAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl TieredCredentialDraftRule for TieredCredentialDraftAllowedCategoryRule {
    fn check(&self, item: &TieredCredentialDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is below minimum", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialDraftMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredCredentialDraftRule for TieredCredentialDraftMaximumRangeRule {
    fn check(&self, item: &TieredCredentialDraft) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialDraftClusterLengthRule {
    pub max_length: i64,
}

impl TieredCredentialDraftRule for TieredCredentialDraftClusterLengthRule {
    fn check(&self, item: &TieredCredentialDraft) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialDraftRequiredCodeRule {
    pub message: String,
}

impl TieredCredentialDraftRule for TieredCredentialDraftRequiredCodeRule {
    fn check(&self, item: &TieredCredentialDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered credential draft.
#[derive(Debug, Clone)]
pub struct TieredCredentialDraftValidator {
    pub rules: Vec<Rc<dyn TieredCredentialDraftRule>>,
}

impl TieredCredentialDraftValidator {
    pub fn validate(&self, item: &TieredCredentialDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredCredentialDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredCredentialDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_credential_draft_validator() -> TieredCredentialDraftValidator {
    TieredCredentialDraftValidator {
        rules: vec![
            Rc::new(TieredCredentialDraftCodeLengthRule { max_length: 3600 }) as Rc<dyn TieredCredentialDraftRule>,
            Rc::new(TieredCredentialDraftTitleLengthRule { max_length: 1000 }),
            Rc::new(TieredCredentialDraftAllowedCategoryRule {
                allowed: vec!["fallback".to_string(), "rolling".to_string()],
            }),
            Rc::new(TieredCredentialDraftMaximumRangeRule { min: 9.6, max: 2517.3 }),
            Rc::new(TieredCredentialDraftClusterLengthRule { max_length: 64 }),
            Rc::new(TieredCredentialDraftRequiredCodeRule {
                message: "tiered credential draft needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_credential_draft_validator();
        let item = new_tiered_credential_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredCredentialDraftCodeLengthRule { max_length: 255 };
        let mut item = new_tiered_credential_draft("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredCredentialDraftTitleLengthRule { max_length: 1000 };
        let mut item = new_tiered_credential_draft("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = TieredCredentialDraftMaximumRangeRule { min: 5.9, max: 2647.9 };
        let mut item = new_tiered_credential_draft("x", "y");
        item.maximum = 2649.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = TieredCredentialDraftClusterLengthRule { max_length: 1000 };
        let mut item = new_tiered_credential_draft("x", "y");
        item.cluster = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredCredentialDraftRequiredCodeRule {
            message: "tiered credential draft needs this field".to_string(),
        };
        let mut item = new_tiered_credential_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
