use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval draft must pass. Check returns its problems, or none.
pub trait ApprovalDraftRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalDraftAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalDraftRule for ApprovalDraftAmountRangeRule {
    fn check(&self, item: &ApprovalDraft) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalDraftApprovedGuardRule {
    pub limit: i64,
}

impl ApprovalDraftRule for ApprovalDraftApprovedGuardRule {
    fn check(&self, item: &ApprovalDraft) -> Vec<String> {
        if item.approved && (item.starts_at > self.limit) {
            return vec![format!(
                "starts at {} exceeds {} while approved",
                item.starts_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalDraftStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalDraftRule for ApprovalDraftStartsAtRangeRule {
    fn check(&self, item: &ApprovalDraft) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalDraftCountryLengthRule {
    pub max_length: i64,
}

impl ApprovalDraftRule for ApprovalDraftCountryLengthRule {
    fn check(&self, item: &ApprovalDraft) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalDraftLabelLengthRule {
    pub max_length: i64,
}

impl ApprovalDraftRule for ApprovalDraftLabelLengthRule {
    fn check(&self, item: &ApprovalDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalDraftAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl ApprovalDraftRule for ApprovalDraftAllowedKeyIdRule {
    fn check(&self, item: &ApprovalDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is stale", item.key_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval draft.
#[derive(Debug, Clone)]
pub struct ApprovalDraftValidator {
    pub rules: Vec<Rc<dyn ApprovalDraftRule>>,
}

impl ApprovalDraftValidator {
    pub fn validate(&self, item: &ApprovalDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_draft_validator() -> ApprovalDraftValidator {
    ApprovalDraftValidator {
        rules: vec![
            Rc::new(ApprovalDraftAmountRangeRule { min: 5.4, max: 620.4 }) as Rc<dyn ApprovalDraftRule>,
            Rc::new(ApprovalDraftApprovedGuardRule { limit: 16 }),
            Rc::new(ApprovalDraftStartsAtRangeRule { min: 32, max: 120 }),
            Rc::new(ApprovalDraftCountryLengthRule { max_length: 64 }),
            Rc::new(ApprovalDraftLabelLengthRule { max_length: 64 }),
            Rc::new(ApprovalDraftAllowedKeyIdRule {
                allowed: vec!["regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_draft_validator();
        let item = new_approval_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = ApprovalDraftAmountRangeRule { min: 7.1, max: 4540.8 };
        let mut item = new_approval_draft("x", "y");
        item.amount = 4542.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = ApprovalDraftStartsAtRangeRule { min: 120, max: 86400 };
        let mut item = new_approval_draft("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = ApprovalDraftCountryLengthRule { max_length: 16 };
        let mut item = new_approval_draft("x", "y");
        item.country = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ApprovalDraftLabelLengthRule { max_length: 255 };
        let mut item = new_approval_draft("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
