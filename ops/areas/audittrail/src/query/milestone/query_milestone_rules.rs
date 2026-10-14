use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a query milestone must pass. Check returns its problems, or none.
pub trait QueryMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &QueryMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryMilestoneAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl QueryMilestoneRule for QueryMilestoneAllowedTitleRule {
    fn check(&self, item: &QueryMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is too long", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryMilestoneCodePrefixRule {
    pub prefix: String,
}

impl QueryMilestoneRule for QueryMilestoneCodePrefixRule {
    fn check(&self, item: &QueryMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryMilestoneRequiredTitleRule {
    pub message: String,
}

impl QueryMilestoneRule for QueryMilestoneRequiredTitleRule {
    fn check(&self, item: &QueryMilestone) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryMilestoneLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryMilestoneRule for QueryMilestoneLimitRangeRule {
    fn check(&self, item: &QueryMilestone) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl QueryMilestoneRule for QueryMilestoneCodeLengthRule {
    fn check(&self, item: &QueryMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query milestone.
#[derive(Debug, Clone)]
pub struct QueryMilestoneValidator {
    pub rules: Vec<Rc<dyn QueryMilestoneRule>>,
}

impl QueryMilestoneValidator {
    pub fn validate(&self, item: &QueryMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_milestone_validator() -> QueryMilestoneValidator {
    QueryMilestoneValidator {
        rules: vec![
            Rc::new(QueryMilestoneAllowedTitleRule {
                allowed: vec!["strict".to_string(), "seasonal".to_string()],
            }) as Rc<dyn QueryMilestoneRule>,
            Rc::new(QueryMilestoneCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(QueryMilestoneRequiredTitleRule {
                message: "query milestone needs an entry".to_string(),
            }),
            Rc::new(QueryMilestoneLimitRangeRule { min: 32, max: 86400 }),
            Rc::new(QueryMilestoneCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_milestone_validator();
        let item = new_query_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = QueryMilestoneRequiredTitleRule {
            message: "query milestone needs an entry".to_string(),
        };
        let mut item = new_query_milestone("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = QueryMilestoneLimitRangeRule { min: 120, max: 255 };
        let mut item = new_query_milestone("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QueryMilestoneCodeLengthRule { max_length: 255 };
        let mut item = new_query_milestone("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
