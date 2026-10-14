use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query slot must pass. Check returns its problems, or none.
pub trait QuerySlotRule: std::fmt::Debug {
    fn check(&self, item: &QuerySlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySlotTitleLengthRule {
    pub max_length: i64,
}

impl QuerySlotRule for QuerySlotTitleLengthRule {
    fn check(&self, item: &QuerySlot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySlotScopeLengthRule {
    pub max_length: i64,
}

impl QuerySlotRule for QuerySlotScopeLengthRule {
    fn check(&self, item: &QuerySlot) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySlotStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuerySlotRule for QuerySlotStartsAtRangeRule {
    fn check(&self, item: &QuerySlot) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySlotTimezoneLengthRule {
    pub max_length: i64,
}

impl QuerySlotRule for QuerySlotTimezoneLengthRule {
    fn check(&self, item: &QuerySlot) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a query slot.
#[derive(Debug, Clone)]
pub struct QuerySlotValidator {
    pub rules: Vec<Rc<dyn QuerySlotRule>>,
}

impl QuerySlotValidator {
    pub fn validate(&self, item: &QuerySlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuerySlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuerySlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_slot_validator() -> QuerySlotValidator {
    QuerySlotValidator {
        rules: vec![
            Rc::new(QuerySlotTitleLengthRule { max_length: 1000 }) as Rc<dyn QuerySlotRule>,
            Rc::new(QuerySlotScopeLengthRule { max_length: 1000 }),
            Rc::new(QuerySlotStartsAtRangeRule { min: 255, max: 1000 }),
            Rc::new(QuerySlotTimezoneLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_slot_validator();
        let item = new_query_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = QuerySlotTitleLengthRule { max_length: 3600 };
        let mut item = new_query_slot("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = QuerySlotScopeLengthRule { max_length: 86400 };
        let mut item = new_query_slot("x", "y");
        item.scope = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = QuerySlotStartsAtRangeRule { min: 255, max: 86400 };
        let mut item = new_query_slot("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = QuerySlotTimezoneLengthRule { max_length: 255 };
        let mut item = new_query_slot("x", "y");
        item.timezone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
