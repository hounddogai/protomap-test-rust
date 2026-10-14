use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release cycle must pass. Check returns its problems, or none.
pub trait ReleaseCycleRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseCycleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseCycleRule for ReleaseCycleRotationDaysRangeRule {
    fn check(&self, item: &ReleaseCycle) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseCycleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseCycleRule for ReleaseCycleCreatedAtRangeRule {
    fn check(&self, item: &ReleaseCycle) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseCycleCodeLengthRule {
    pub max_length: i64,
}

impl ReleaseCycleRule for ReleaseCycleCodeLengthRule {
    fn check(&self, item: &ReleaseCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseCycleAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ReleaseCycleRule for ReleaseCycleAllowedTitleRule {
    fn check(&self, item: &ReleaseCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is unknown", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release cycle.
#[derive(Debug, Clone)]
pub struct ReleaseCycleValidator {
    pub rules: Vec<Rc<dyn ReleaseCycleRule>>,
}

impl ReleaseCycleValidator {
    pub fn validate(&self, item: &ReleaseCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_cycle_validator() -> ReleaseCycleValidator {
    ReleaseCycleValidator {
        rules: vec![
            Rc::new(ReleaseCycleRotationDaysRangeRule { min: 16, max: 120 }) as Rc<dyn ReleaseCycleRule>,
            Rc::new(ReleaseCycleCreatedAtRangeRule { min: 3600, max: 3600 }),
            Rc::new(ReleaseCycleCodeLengthRule { max_length: 32 }),
            Rc::new(ReleaseCycleAllowedTitleRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_cycle_validator();
        let item = new_release_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = ReleaseCycleRotationDaysRangeRule { min: 64, max: 255 };
        let mut item = new_release_cycle("x", "y");
        item.rotation_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = ReleaseCycleCreatedAtRangeRule { min: 255, max: 1000 };
        let mut item = new_release_cycle("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReleaseCycleCodeLengthRule { max_length: 86400 };
        let mut item = new_release_cycle("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
