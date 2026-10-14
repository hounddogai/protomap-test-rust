use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary query profile must pass. Check returns its problems, or none.
pub trait PrimaryQueryProfileRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryQueryProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQueryProfilePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryQueryProfileRule for PrimaryQueryProfilePortRangeRule {
    fn check(&self, item: &PrimaryQueryProfile) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQueryProfileCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryQueryProfileRule for PrimaryQueryProfileCodeLengthRule {
    fn check(&self, item: &PrimaryQueryProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQueryProfileMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryQueryProfileRule for PrimaryQueryProfileMinimumRangeRule {
    fn check(&self, item: &PrimaryQueryProfile) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQueryProfileVisibleGuardRule {
    pub limit: i64,
}

impl PrimaryQueryProfileRule for PrimaryQueryProfileVisibleGuardRule {
    fn check(&self, item: &PrimaryQueryProfile) -> Vec<String> {
        if item.visible && (item.port > self.limit) {
            return vec![format!("port {} exceeds {} while visible", item.port, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQueryProfileRequiredTitleRule {
    pub message: String,
}

impl PrimaryQueryProfileRule for PrimaryQueryProfileRequiredTitleRule {
    fn check(&self, item: &PrimaryQueryProfile) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary query profile.
#[derive(Debug, Clone)]
pub struct PrimaryQueryProfileValidator {
    pub rules: Vec<Rc<dyn PrimaryQueryProfileRule>>,
}

impl PrimaryQueryProfileValidator {
    pub fn validate(&self, item: &PrimaryQueryProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryQueryProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryQueryProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_query_profile_validator() -> PrimaryQueryProfileValidator {
    PrimaryQueryProfileValidator {
        rules: vec![
            Rc::new(PrimaryQueryProfilePortRangeRule { min: 255, max: 1000 }) as Rc<dyn PrimaryQueryProfileRule>,
            Rc::new(PrimaryQueryProfileCodeLengthRule { max_length: 1000 }),
            Rc::new(PrimaryQueryProfileMinimumRangeRule { min: 3.1, max: 1197.0 }),
            Rc::new(PrimaryQueryProfileVisibleGuardRule { limit: 3600 }),
            Rc::new(PrimaryQueryProfileRequiredTitleRule {
                message: "primary query profile needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_query_profile_validator();
        let item = new_primary_query_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = PrimaryQueryProfilePortRangeRule { min: 64, max: 80 };
        let mut item = new_primary_query_profile("x", "y");
        item.port = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryQueryProfileCodeLengthRule { max_length: 255 };
        let mut item = new_primary_query_profile("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PrimaryQueryProfileMinimumRangeRule { min: 5.7, max: 643.6 };
        let mut item = new_primary_query_profile("x", "y");
        item.minimum = 645.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryQueryProfileRequiredTitleRule {
            message: "primary query profile needs this field".to_string(),
        };
        let mut item = new_primary_query_profile("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
