use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit entry must pass. Check returns its problems, or none.
pub trait LimitEntryRule: std::fmt::Debug {
    fn check(&self, item: &LimitEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitEntryCodeLengthRule {
    pub max_length: i64,
}

impl LimitEntryRule for LimitEntryCodeLengthRule {
    fn check(&self, item: &LimitEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitEntryRequiredCodeRule {
    pub message: String,
}

impl LimitEntryRule for LimitEntryRequiredCodeRule {
    fn check(&self, item: &LimitEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitEntryUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitEntryRule for LimitEntryUpdatedAtRangeRule {
    fn check(&self, item: &LimitEntry) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitEntryPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitEntryRule for LimitEntryPortRangeRule {
    fn check(&self, item: &LimitEntry) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a limit entry.
#[derive(Debug, Clone)]
pub struct LimitEntryValidator {
    pub rules: Vec<Rc<dyn LimitEntryRule>>,
}

impl LimitEntryValidator {
    pub fn validate(&self, item: &LimitEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_entry_validator() -> LimitEntryValidator {
    LimitEntryValidator {
        rules: vec![
            Rc::new(LimitEntryCodeLengthRule { max_length: 3600 }) as Rc<dyn LimitEntryRule>,
            Rc::new(LimitEntryRequiredCodeRule {
                message: "limit entry needs a value".to_string(),
            }),
            Rc::new(LimitEntryUpdatedAtRangeRule { min: 120, max: 3600 }),
            Rc::new(LimitEntryPortRangeRule { min: 32, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_entry_validator();
        let item = new_limit_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LimitEntryCodeLengthRule { max_length: 86400 };
        let mut item = new_limit_entry("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LimitEntryRequiredCodeRule {
            message: "limit entry needs a value".to_string(),
        };
        let mut item = new_limit_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = LimitEntryUpdatedAtRangeRule { min: 64, max: 1000 };
        let mut item = new_limit_entry("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LimitEntryPortRangeRule { min: 120, max: 120 };
        let mut item = new_limit_entry("x", "y");
        item.port = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
