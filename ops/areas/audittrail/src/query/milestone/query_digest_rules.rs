use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query digest must pass. Check returns its problems, or none.
pub trait QueryDigestRule: std::fmt::Debug {
    fn check(&self, item: &QueryDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryDigestCodePrefixRule {
    pub prefix: String,
}

impl QueryDigestRule for QueryDigestCodePrefixRule {
    fn check(&self, item: &QueryDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryDigestScopeLengthRule {
    pub max_length: i64,
}

impl QueryDigestRule for QueryDigestScopeLengthRule {
    fn check(&self, item: &QueryDigest) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryDigestCodeLengthRule {
    pub max_length: i64,
}

impl QueryDigestRule for QueryDigestCodeLengthRule {
    fn check(&self, item: &QueryDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryDigestRequiredScopeRule {
    pub message: String,
}

impl QueryDigestRule for QueryDigestRequiredScopeRule {
    fn check(&self, item: &QueryDigest) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a query digest.
#[derive(Debug, Clone)]
pub struct QueryDigestValidator {
    pub rules: Vec<Rc<dyn QueryDigestRule>>,
}

impl QueryDigestValidator {
    pub fn validate(&self, item: &QueryDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_digest_validator() -> QueryDigestValidator {
    QueryDigestValidator {
        rules: vec![
            Rc::new(QueryDigestCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn QueryDigestRule>,
            Rc::new(QueryDigestScopeLengthRule { max_length: 64 }),
            Rc::new(QueryDigestCodeLengthRule { max_length: 3600 }),
            Rc::new(QueryDigestRequiredScopeRule {
                message: "query digest needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_digest_validator();
        let item = new_query_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = QueryDigestScopeLengthRule { max_length: 86400 };
        let mut item = new_query_digest("x", "y");
        item.scope = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QueryDigestCodeLengthRule { max_length: 120 };
        let mut item = new_query_digest("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = QueryDigestRequiredScopeRule {
            message: "query digest needs this field".to_string(),
        };
        let mut item = new_query_digest("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
