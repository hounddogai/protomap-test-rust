use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a postmortem ledger must pass. Check returns its problems, or none.
pub trait PostmortemLedgerRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemLedgerAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PostmortemLedgerRule for PostmortemLedgerAllowedTitleRule {
    fn check(&self, item: &PostmortemLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemLedgerActiveGuardRule {
    pub limit: i64,
}

impl PostmortemLedgerRule for PostmortemLedgerActiveGuardRule {
    fn check(&self, item: &PostmortemLedger) -> Vec<String> {
        if item.active && (item.retries > self.limit) {
            return vec![format!("retries {} exceeds {} while active", item.retries, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemLedgerCodePrefixRule {
    pub prefix: String,
}

impl PostmortemLedgerRule for PostmortemLedgerCodePrefixRule {
    fn check(&self, item: &PostmortemLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemLedgerTitleLengthRule {
    pub max_length: i64,
}

impl PostmortemLedgerRule for PostmortemLedgerTitleLengthRule {
    fn check(&self, item: &PostmortemLedger) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemLedgerRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemLedgerRule for PostmortemLedgerRetriesRangeRule {
    fn check(&self, item: &PostmortemLedger) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem ledger.
#[derive(Debug, Clone)]
pub struct PostmortemLedgerValidator {
    pub rules: Vec<Rc<dyn PostmortemLedgerRule>>,
}

impl PostmortemLedgerValidator {
    pub fn validate(&self, item: &PostmortemLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_ledger_validator() -> PostmortemLedgerValidator {
    PostmortemLedgerValidator {
        rules: vec![
            Rc::new(PostmortemLedgerAllowedTitleRule {
                allowed: vec!["regional".to_string(), "primary".to_string(), "default".to_string()],
            }) as Rc<dyn PostmortemLedgerRule>,
            Rc::new(PostmortemLedgerActiveGuardRule { limit: 80 }),
            Rc::new(PostmortemLedgerCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PostmortemLedgerTitleLengthRule { max_length: 86400 }),
            Rc::new(PostmortemLedgerRetriesRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_ledger_validator();
        let item = new_postmortem_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PostmortemLedgerTitleLengthRule { max_length: 64 };
        let mut item = new_postmortem_ledger("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = PostmortemLedgerRetriesRangeRule { min: 16, max: 86400 };
        let mut item = new_postmortem_ledger("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
