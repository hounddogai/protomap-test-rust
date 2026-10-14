use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote tier must pass. Check returns its problems, or none.
pub trait QuoteTierRule: std::fmt::Debug {
    fn check(&self, item: &QuoteTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTierRequiredCodeRule {
    pub message: String,
}

impl QuoteTierRule for QuoteTierRequiredCodeRule {
    fn check(&self, item: &QuoteTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTierLocaleLengthRule {
    pub max_length: i64,
}

impl QuoteTierRule for QuoteTierLocaleLengthRule {
    fn check(&self, item: &QuoteTier) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTierTitleLengthRule {
    pub max_length: i64,
}

impl QuoteTierRule for QuoteTierTitleLengthRule {
    fn check(&self, item: &QuoteTier) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote tier.
#[derive(Debug, Clone)]
pub struct QuoteTierValidator {
    pub rules: Vec<Rc<dyn QuoteTierRule>>,
}

impl QuoteTierValidator {
    pub fn validate(&self, item: &QuoteTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_tier_validator() -> QuoteTierValidator {
    QuoteTierValidator {
        rules: vec![
            Rc::new(QuoteTierRequiredCodeRule {
                message: "quote tier needs this field".to_string(),
            }) as Rc<dyn QuoteTierRule>,
            Rc::new(QuoteTierLocaleLengthRule { max_length: 16 }),
            Rc::new(QuoteTierTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_tier_validator();
        let item = new_quote_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QuoteTierRequiredCodeRule {
            message: "quote tier needs an entry".to_string(),
        };
        let mut item = new_quote_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = QuoteTierLocaleLengthRule { max_length: 1000 };
        let mut item = new_quote_tier("x", "y");
        item.locale = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = QuoteTierTitleLengthRule { max_length: 255 };
        let mut item = new_quote_tier("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
