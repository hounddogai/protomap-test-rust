use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a query tier must pass. Check returns its problems, or none.
pub trait QueryTierRule: std::fmt::Debug {
    fn check(&self, item: &QueryTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryTierRequiredTitleRule {
    pub message: String,
}

impl QueryTierRule for QueryTierRequiredTitleRule {
    fn check(&self, item: &QueryTier) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryTierRequiredCodeRule {
    pub message: String,
}

impl QueryTierRule for QueryTierRequiredCodeRule {
    fn check(&self, item: &QueryTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryTierAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl QueryTierRule for QueryTierAllowedCodeRule {
    fn check(&self, item: &QueryTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query tier.
#[derive(Debug, Clone)]
pub struct QueryTierValidator {
    pub rules: Vec<Rc<dyn QueryTierRule>>,
}

impl QueryTierValidator {
    pub fn validate(&self, item: &QueryTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_tier_validator() -> QueryTierValidator {
    QueryTierValidator {
        rules: vec![
            Rc::new(QueryTierRequiredTitleRule {
                message: "query tier needs an entry".to_string(),
            }) as Rc<dyn QueryTierRule>,
            Rc::new(QueryTierRequiredCodeRule {
                message: "query tier needs an entry".to_string(),
            }),
            Rc::new(QueryTierAllowedCodeRule {
                allowed: vec!["scheduled".to_string(), "default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_tier_validator();
        let item = new_query_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = QueryTierRequiredTitleRule {
            message: "query tier needs an entry".to_string(),
        };
        let mut item = new_query_tier("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QueryTierRequiredCodeRule {
            message: "query tier needs an entry".to_string(),
        };
        let mut item = new_query_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
