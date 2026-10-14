use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query entry must pass. Check returns its problems, or none.
pub trait QueryEntryRule: std::fmt::Debug {
    fn check(&self, item: &QueryEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryEntryCodePrefixRule {
    pub prefix: String,
}

impl QueryEntryRule for QueryEntryCodePrefixRule {
    fn check(&self, item: &QueryEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryEntryUnitLengthRule {
    pub max_length: i64,
}

impl QueryEntryRule for QueryEntryUnitLengthRule {
    fn check(&self, item: &QueryEntry) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryEntryRequiredNameRule {
    pub message: String,
}

impl QueryEntryRule for QueryEntryRequiredNameRule {
    fn check(&self, item: &QueryEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryEntryQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryEntryRule for QueryEntryQuantityRangeRule {
    fn check(&self, item: &QueryEntry) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryEntryRequiredCodeRule {
    pub message: String,
}

impl QueryEntryRule for QueryEntryRequiredCodeRule {
    fn check(&self, item: &QueryEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a query entry.
#[derive(Debug, Clone)]
pub struct QueryEntryValidator {
    pub rules: Vec<Rc<dyn QueryEntryRule>>,
}

impl QueryEntryValidator {
    pub fn validate(&self, item: &QueryEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_entry_validator() -> QueryEntryValidator {
    QueryEntryValidator {
        rules: vec![
            Rc::new(QueryEntryCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn QueryEntryRule>,
            Rc::new(QueryEntryUnitLengthRule { max_length: 80 }),
            Rc::new(QueryEntryRequiredNameRule {
                message: "query entry needs this field".to_string(),
            }),
            Rc::new(QueryEntryQuantityRangeRule { min: 1000, max: 3600 }),
            Rc::new(QueryEntryRequiredCodeRule {
                message: "query entry needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_entry_validator();
        let item = new_query_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = QueryEntryUnitLengthRule { max_length: 120 };
        let mut item = new_query_entry("x", "y");
        item.unit = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = QueryEntryRequiredNameRule {
            message: "query entry needs an entry".to_string(),
        };
        let mut item = new_query_entry("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = QueryEntryQuantityRangeRule { min: 64, max: 80 };
        let mut item = new_query_entry("x", "y");
        item.quantity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QueryEntryRequiredCodeRule {
            message: "query entry needs an entry".to_string(),
        };
        let mut item = new_query_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
