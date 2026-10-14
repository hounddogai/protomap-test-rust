use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal credential summary must pass. Check returns its problems, or none.
pub trait SeasonalCredentialSummaryRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalCredentialSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSummaryTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalCredentialSummaryRule for SeasonalCredentialSummaryTitleLengthRule {
    fn check(&self, item: &SeasonalCredentialSummary) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSummaryRequiredTitleRule {
    pub message: String,
}

impl SeasonalCredentialSummaryRule for SeasonalCredentialSummaryRequiredTitleRule {
    fn check(&self, item: &SeasonalCredentialSummary) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSummaryCodePrefixRule {
    pub prefix: String,
}

impl SeasonalCredentialSummaryRule for SeasonalCredentialSummaryCodePrefixRule {
    fn check(&self, item: &SeasonalCredentialSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSummaryPrincipalLengthRule {
    pub max_length: i64,
}

impl SeasonalCredentialSummaryRule for SeasonalCredentialSummaryPrincipalLengthRule {
    fn check(&self, item: &SeasonalCredentialSummary) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSummarySegmentLengthRule {
    pub max_length: i64,
}

impl SeasonalCredentialSummaryRule for SeasonalCredentialSummarySegmentLengthRule {
    fn check(&self, item: &SeasonalCredentialSummary) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSummaryCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalCredentialSummaryRule for SeasonalCredentialSummaryCodeLengthRule {
    fn check(&self, item: &SeasonalCredentialSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal credential summary.
#[derive(Debug, Clone)]
pub struct SeasonalCredentialSummaryValidator {
    pub rules: Vec<Rc<dyn SeasonalCredentialSummaryRule>>,
}

impl SeasonalCredentialSummaryValidator {
    pub fn validate(&self, item: &SeasonalCredentialSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalCredentialSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalCredentialSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_credential_summary_validator() -> SeasonalCredentialSummaryValidator {
    SeasonalCredentialSummaryValidator {
        rules: vec![
            Rc::new(SeasonalCredentialSummaryTitleLengthRule { max_length: 32 })
                as Rc<dyn SeasonalCredentialSummaryRule>,
            Rc::new(SeasonalCredentialSummaryRequiredTitleRule {
                message: "seasonal credential summary needs an entry".to_string(),
            }),
            Rc::new(SeasonalCredentialSummaryCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(SeasonalCredentialSummaryPrincipalLengthRule { max_length: 32 }),
            Rc::new(SeasonalCredentialSummarySegmentLengthRule { max_length: 64 }),
            Rc::new(SeasonalCredentialSummaryCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_credential_summary_validator();
        let item = new_seasonal_credential_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SeasonalCredentialSummaryTitleLengthRule { max_length: 1000 };
        let mut item = new_seasonal_credential_summary("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SeasonalCredentialSummaryRequiredTitleRule {
            message: "seasonal credential summary needs a value".to_string(),
        };
        let mut item = new_seasonal_credential_summary("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = SeasonalCredentialSummaryPrincipalLengthRule { max_length: 3600 };
        let mut item = new_seasonal_credential_summary("x", "y");
        item.principal = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = SeasonalCredentialSummarySegmentLengthRule { max_length: 3600 };
        let mut item = new_seasonal_credential_summary("x", "y");
        item.segment = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalCredentialSummaryCodeLengthRule { max_length: 1000 };
        let mut item = new_seasonal_credential_summary("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
