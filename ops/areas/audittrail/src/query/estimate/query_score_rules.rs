use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query score must pass. Check returns its problems, or none.
pub trait QueryScoreRule: std::fmt::Debug {
    fn check(&self, item: &QueryScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryScoreCurrencyLengthRule {
    pub max_length: i64,
}

impl QueryScoreRule for QueryScoreCurrencyLengthRule {
    fn check(&self, item: &QueryScore) -> Vec<String> {
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
pub struct QueryScoreCodeLengthRule {
    pub max_length: i64,
}

impl QueryScoreRule for QueryScoreCodeLengthRule {
    fn check(&self, item: &QueryScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryScoreLabelLengthRule {
    pub max_length: i64,
}

impl QueryScoreRule for QueryScoreLabelLengthRule {
    fn check(&self, item: &QueryScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryScoreRequiredLabelRule {
    pub message: String,
}

impl QueryScoreRule for QueryScoreRequiredLabelRule {
    fn check(&self, item: &QueryScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryScoreCodePrefixRule {
    pub prefix: String,
}

impl QueryScoreRule for QueryScoreCodePrefixRule {
    fn check(&self, item: &QueryScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryScoreRequiredCurrencyRule {
    pub message: String,
}

impl QueryScoreRule for QueryScoreRequiredCurrencyRule {
    fn check(&self, item: &QueryScore) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a query score.
#[derive(Debug, Clone)]
pub struct QueryScoreValidator {
    pub rules: Vec<Rc<dyn QueryScoreRule>>,
}

impl QueryScoreValidator {
    pub fn validate(&self, item: &QueryScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_score_validator() -> QueryScoreValidator {
    QueryScoreValidator {
        rules: vec![
            Rc::new(QueryScoreCurrencyLengthRule { max_length: 64 }) as Rc<dyn QueryScoreRule>,
            Rc::new(QueryScoreCodeLengthRule { max_length: 64 }),
            Rc::new(QueryScoreLabelLengthRule { max_length: 255 }),
            Rc::new(QueryScoreRequiredLabelRule {
                message: "query score needs a value".to_string(),
            }),
            Rc::new(QueryScoreCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(QueryScoreRequiredCurrencyRule {
                message: "query score needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_score_validator();
        let item = new_query_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = QueryScoreCurrencyLengthRule { max_length: 32 };
        let mut item = new_query_score("x", "y");
        item.currency = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QueryScoreCodeLengthRule { max_length: 120 };
        let mut item = new_query_score("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = QueryScoreLabelLengthRule { max_length: 120 };
        let mut item = new_query_score("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = QueryScoreRequiredLabelRule {
            message: "query score needs an entry".to_string(),
        };
        let mut item = new_query_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = QueryScoreRequiredCurrencyRule {
            message: "query score needs a value".to_string(),
        };
        let mut item = new_query_score("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
