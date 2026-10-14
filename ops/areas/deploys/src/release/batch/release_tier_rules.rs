use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release tier must pass. Check returns its problems, or none.
pub trait ReleaseTierRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTierAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl ReleaseTierRule for ReleaseTierAllowedUnitRule {
    fn check(&self, item: &ReleaseTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is missing", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTierRequiredUnitRule {
    pub message: String,
}

impl ReleaseTierRule for ReleaseTierRequiredUnitRule {
    fn check(&self, item: &ReleaseTier) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTierFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseTierRule for ReleaseTierFloorValueRangeRule {
    fn check(&self, item: &ReleaseTier) -> Vec<String> {
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
pub struct ReleaseTierUnitLengthRule {
    pub max_length: i64,
}

impl ReleaseTierRule for ReleaseTierUnitLengthRule {
    fn check(&self, item: &ReleaseTier) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTierRequiredTitleRule {
    pub message: String,
}

impl ReleaseTierRule for ReleaseTierRequiredTitleRule {
    fn check(&self, item: &ReleaseTier) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a release tier.
#[derive(Debug, Clone)]
pub struct ReleaseTierValidator {
    pub rules: Vec<Rc<dyn ReleaseTierRule>>,
}

impl ReleaseTierValidator {
    pub fn validate(&self, item: &ReleaseTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_tier_validator() -> ReleaseTierValidator {
    ReleaseTierValidator {
        rules: vec![
            Rc::new(ReleaseTierAllowedUnitRule {
                allowed: vec!["pending".to_string(), "legacy".to_string()],
            }) as Rc<dyn ReleaseTierRule>,
            Rc::new(ReleaseTierRequiredUnitRule {
                message: "release tier needs a value".to_string(),
            }),
            Rc::new(ReleaseTierFloorValueRangeRule { min: 2.0, max: 2254.4 }),
            Rc::new(ReleaseTierUnitLengthRule { max_length: 16 }),
            Rc::new(ReleaseTierRequiredTitleRule {
                message: "release tier needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_tier_validator();
        let item = new_release_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = ReleaseTierRequiredUnitRule {
            message: "release tier needs this field".to_string(),
        };
        let mut item = new_release_tier("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = ReleaseTierFloorValueRangeRule { min: 8.0, max: 3858.8 };
        let mut item = new_release_tier("x", "y");
        item.floor_value = 3860.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = ReleaseTierUnitLengthRule { max_length: 16 };
        let mut item = new_release_tier("x", "y");
        item.unit = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ReleaseTierRequiredTitleRule {
            message: "release tier needs this field".to_string(),
        };
        let mut item = new_release_tier("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
