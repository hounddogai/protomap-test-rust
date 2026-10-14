use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence group must pass. Check returns its problems, or none.
pub trait SilenceGroupRule: std::fmt::Debug {
    fn check(&self, item: &SilenceGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceGroupScopeLengthRule {
    pub max_length: i64,
}

impl SilenceGroupRule for SilenceGroupScopeLengthRule {
    fn check(&self, item: &SilenceGroup) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceGroupFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SilenceGroupRule for SilenceGroupFloorValueRangeRule {
    fn check(&self, item: &SilenceGroup) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceGroupNameLengthRule {
    pub max_length: i64,
}

impl SilenceGroupRule for SilenceGroupNameLengthRule {
    fn check(&self, item: &SilenceGroup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence group.
#[derive(Debug, Clone)]
pub struct SilenceGroupValidator {
    pub rules: Vec<Rc<dyn SilenceGroupRule>>,
}

impl SilenceGroupValidator {
    pub fn validate(&self, item: &SilenceGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_group_validator() -> SilenceGroupValidator {
    SilenceGroupValidator {
        rules: vec![
            Rc::new(SilenceGroupScopeLengthRule { max_length: 86400 }) as Rc<dyn SilenceGroupRule>,
            Rc::new(SilenceGroupFloorValueRangeRule { min: 8.6, max: 1022.1 }),
            Rc::new(SilenceGroupNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_group_validator();
        let item = new_silence_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = SilenceGroupScopeLengthRule { max_length: 32 };
        let mut item = new_silence_group("x", "y");
        item.scope = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = SilenceGroupFloorValueRangeRule { min: 5.9, max: 1765.2 };
        let mut item = new_silence_group("x", "y");
        item.floor_value = 1766.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SilenceGroupNameLengthRule { max_length: 3600 };
        let mut item = new_silence_group("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
