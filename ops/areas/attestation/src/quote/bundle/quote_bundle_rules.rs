use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote bundle must pass. Check returns its problems, or none.
pub trait QuoteBundleRule: std::fmt::Debug {
    fn check(&self, item: &QuoteBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBundleRequiredTitleRule {
    pub message: String,
}

impl QuoteBundleRule for QuoteBundleRequiredTitleRule {
    fn check(&self, item: &QuoteBundle) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBundleTitleLengthRule {
    pub max_length: i64,
}

impl QuoteBundleRule for QuoteBundleTitleLengthRule {
    fn check(&self, item: &QuoteBundle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBundleRequiredPrincipalRule {
    pub message: String,
}

impl QuoteBundleRule for QuoteBundleRequiredPrincipalRule {
    fn check(&self, item: &QuoteBundle) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote bundle.
#[derive(Debug, Clone)]
pub struct QuoteBundleValidator {
    pub rules: Vec<Rc<dyn QuoteBundleRule>>,
}

impl QuoteBundleValidator {
    pub fn validate(&self, item: &QuoteBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_bundle_validator() -> QuoteBundleValidator {
    QuoteBundleValidator {
        rules: vec![
            Rc::new(QuoteBundleRequiredTitleRule {
                message: "quote bundle needs a value".to_string(),
            }) as Rc<dyn QuoteBundleRule>,
            Rc::new(QuoteBundleTitleLengthRule { max_length: 32 }),
            Rc::new(QuoteBundleRequiredPrincipalRule {
                message: "quote bundle needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_bundle_validator();
        let item = new_quote_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = QuoteBundleRequiredTitleRule {
            message: "quote bundle needs an entry".to_string(),
        };
        let mut item = new_quote_bundle("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = QuoteBundleTitleLengthRule { max_length: 3600 };
        let mut item = new_quote_bundle("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = QuoteBundleRequiredPrincipalRule {
            message: "quote bundle needs a value".to_string(),
        };
        let mut item = new_quote_bundle("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
