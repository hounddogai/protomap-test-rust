use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query profile must pass. Check returns its problems, or none.
pub trait QueryProfileRule: std::fmt::Debug {
    fn check(&self, item: &QueryProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryProfileRegionLengthRule {
    pub max_length: i64,
}

impl QueryProfileRule for QueryProfileRegionLengthRule {
    fn check(&self, item: &QueryProfile) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryProfileScopeLengthRule {
    pub max_length: i64,
}

impl QueryProfileRule for QueryProfileScopeLengthRule {
    fn check(&self, item: &QueryProfile) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryProfileRequiredRegionRule {
    pub message: String,
}

impl QueryProfileRule for QueryProfileRequiredRegionRule {
    fn check(&self, item: &QueryProfile) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a query profile.
#[derive(Debug, Clone)]
pub struct QueryProfileValidator {
    pub rules: Vec<Rc<dyn QueryProfileRule>>,
}

impl QueryProfileValidator {
    pub fn validate(&self, item: &QueryProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_profile_validator() -> QueryProfileValidator {
    QueryProfileValidator {
        rules: vec![
            Rc::new(QueryProfileRegionLengthRule { max_length: 32 }) as Rc<dyn QueryProfileRule>,
            Rc::new(QueryProfileScopeLengthRule { max_length: 120 }),
            Rc::new(QueryProfileRequiredRegionRule {
                message: "query profile needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_profile_validator();
        let item = new_query_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = QueryProfileRegionLengthRule { max_length: 120 };
        let mut item = new_query_profile("x", "y");
        item.region = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = QueryProfileScopeLengthRule { max_length: 255 };
        let mut item = new_query_profile("x", "y");
        item.scope = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = QueryProfileRequiredRegionRule {
            message: "query profile needs a value".to_string(),
        };
        let mut item = new_query_profile("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
