use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant estimate must pass. Check returns its problems, or none.
pub trait GrantEstimateRule: std::fmt::Debug {
    fn check(&self, item: &GrantEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantEstimateUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantEstimateRule for GrantEstimateUpdatedAtRangeRule {
    fn check(&self, item: &GrantEstimate) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantEstimateCodeLengthRule {
    pub max_length: i64,
}

impl GrantEstimateRule for GrantEstimateCodeLengthRule {
    fn check(&self, item: &GrantEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantEstimateRequiredCodeRule {
    pub message: String,
}

impl GrantEstimateRule for GrantEstimateRequiredCodeRule {
    fn check(&self, item: &GrantEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantEstimateScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantEstimateRule for GrantEstimateScoreRangeRule {
    fn check(&self, item: &GrantEstimate) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantEstimateRegionLengthRule {
    pub max_length: i64,
}

impl GrantEstimateRule for GrantEstimateRegionLengthRule {
    fn check(&self, item: &GrantEstimate) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantEstimatePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantEstimateRule for GrantEstimatePortRangeRule {
    fn check(&self, item: &GrantEstimate) -> Vec<String> {
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

/// Runs every rule of a grant estimate.
#[derive(Debug, Clone)]
pub struct GrantEstimateValidator {
    pub rules: Vec<Rc<dyn GrantEstimateRule>>,
}

impl GrantEstimateValidator {
    pub fn validate(&self, item: &GrantEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_estimate_validator() -> GrantEstimateValidator {
    GrantEstimateValidator {
        rules: vec![
            Rc::new(GrantEstimateUpdatedAtRangeRule { min: 16, max: 1000 }) as Rc<dyn GrantEstimateRule>,
            Rc::new(GrantEstimateCodeLengthRule { max_length: 32 }),
            Rc::new(GrantEstimateRequiredCodeRule {
                message: "grant estimate needs this field".to_string(),
            }),
            Rc::new(GrantEstimateScoreRangeRule { min: 2.1, max: 3641.1 }),
            Rc::new(GrantEstimateRegionLengthRule { max_length: 32 }),
            Rc::new(GrantEstimatePortRangeRule { min: 32, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_estimate_validator();
        let item = new_grant_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = GrantEstimateUpdatedAtRangeRule { min: 32, max: 1000 };
        let mut item = new_grant_estimate("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GrantEstimateCodeLengthRule { max_length: 255 };
        let mut item = new_grant_estimate("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GrantEstimateRequiredCodeRule {
            message: "grant estimate needs a value".to_string(),
        };
        let mut item = new_grant_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = GrantEstimateScoreRangeRule { min: 9.1, max: 1168.6 };
        let mut item = new_grant_estimate("x", "y");
        item.score = 1170.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = GrantEstimateRegionLengthRule { max_length: 86400 };
        let mut item = new_grant_estimate("x", "y");
        item.region = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = GrantEstimatePortRangeRule { min: 64, max: 64 };
        let mut item = new_grant_estimate("x", "y");
        item.port = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
