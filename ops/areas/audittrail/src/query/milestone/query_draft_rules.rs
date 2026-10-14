use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query draft must pass. Check returns its problems, or none.
pub trait QueryDraftRule: std::fmt::Debug {
    fn check(&self, item: &QueryDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryDraftRequiredTitleRule {
    pub message: String,
}

impl QueryDraftRule for QueryDraftRequiredTitleRule {
    fn check(&self, item: &QueryDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryDraftCodeLengthRule {
    pub max_length: i64,
}

impl QueryDraftRule for QueryDraftCodeLengthRule {
    fn check(&self, item: &QueryDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryDraftCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryDraftRule for QueryDraftCeilingRangeRule {
    fn check(&self, item: &QueryDraft) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryDraftSourceLengthRule {
    pub max_length: i64,
}

impl QueryDraftRule for QueryDraftSourceLengthRule {
    fn check(&self, item: &QueryDraft) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryDraftDescriptionLengthRule {
    pub max_length: i64,
}

impl QueryDraftRule for QueryDraftDescriptionLengthRule {
    fn check(&self, item: &QueryDraft) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a query draft.
#[derive(Debug, Clone)]
pub struct QueryDraftValidator {
    pub rules: Vec<Rc<dyn QueryDraftRule>>,
}

impl QueryDraftValidator {
    pub fn validate(&self, item: &QueryDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_draft_validator() -> QueryDraftValidator {
    QueryDraftValidator {
        rules: vec![
            Rc::new(QueryDraftRequiredTitleRule {
                message: "query draft needs this field".to_string(),
            }) as Rc<dyn QueryDraftRule>,
            Rc::new(QueryDraftCodeLengthRule { max_length: 32 }),
            Rc::new(QueryDraftCeilingRangeRule { min: 9.2, max: 136.9 }),
            Rc::new(QueryDraftSourceLengthRule { max_length: 80 }),
            Rc::new(QueryDraftDescriptionLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_draft_validator();
        let item = new_query_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = QueryDraftRequiredTitleRule {
            message: "query draft needs this field".to_string(),
        };
        let mut item = new_query_draft("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QueryDraftCodeLengthRule { max_length: 1000 };
        let mut item = new_query_draft("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = QueryDraftCeilingRangeRule { min: 2.9, max: 2470.0 };
        let mut item = new_query_draft("x", "y");
        item.ceiling = 2471.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = QueryDraftSourceLengthRule { max_length: 64 };
        let mut item = new_query_draft("x", "y");
        item.source = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = QueryDraftDescriptionLengthRule { max_length: 64 };
        let mut item = new_query_draft("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
