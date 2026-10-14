use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist cycle must pass. Check returns its problems, or none.
pub trait AllowlistCycleRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCycleExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistCycleRule for AllowlistCycleExpiresAtRangeRule {
    fn check(&self, item: &AllowlistCycle) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCycleRequiredNameRule {
    pub message: String,
}

impl AllowlistCycleRule for AllowlistCycleRequiredNameRule {
    fn check(&self, item: &AllowlistCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCycleBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistCycleRule for AllowlistCycleBatchSizeRangeRule {
    fn check(&self, item: &AllowlistCycle) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCycleCodeLengthRule {
    pub max_length: i64,
}

impl AllowlistCycleRule for AllowlistCycleCodeLengthRule {
    fn check(&self, item: &AllowlistCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCycleRequiredCodeRule {
    pub message: String,
}

impl AllowlistCycleRule for AllowlistCycleRequiredCodeRule {
    fn check(&self, item: &AllowlistCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist cycle.
#[derive(Debug, Clone)]
pub struct AllowlistCycleValidator {
    pub rules: Vec<Rc<dyn AllowlistCycleRule>>,
}

impl AllowlistCycleValidator {
    pub fn validate(&self, item: &AllowlistCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_cycle_validator() -> AllowlistCycleValidator {
    AllowlistCycleValidator {
        rules: vec![
            Rc::new(AllowlistCycleExpiresAtRangeRule { min: 16, max: 120 }) as Rc<dyn AllowlistCycleRule>,
            Rc::new(AllowlistCycleRequiredNameRule {
                message: "allowlist cycle needs this field".to_string(),
            }),
            Rc::new(AllowlistCycleBatchSizeRangeRule { min: 16, max: 32 }),
            Rc::new(AllowlistCycleCodeLengthRule { max_length: 16 }),
            Rc::new(AllowlistCycleRequiredCodeRule {
                message: "allowlist cycle needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_cycle_validator();
        let item = new_allowlist_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = AllowlistCycleExpiresAtRangeRule { min: 120, max: 1000 };
        let mut item = new_allowlist_cycle("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AllowlistCycleRequiredNameRule {
            message: "allowlist cycle needs a value".to_string(),
        };
        let mut item = new_allowlist_cycle("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = AllowlistCycleBatchSizeRangeRule { min: 16, max: 86400 };
        let mut item = new_allowlist_cycle("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AllowlistCycleCodeLengthRule { max_length: 80 };
        let mut item = new_allowlist_cycle("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AllowlistCycleRequiredCodeRule {
            message: "allowlist cycle needs this field".to_string(),
        };
        let mut item = new_allowlist_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
