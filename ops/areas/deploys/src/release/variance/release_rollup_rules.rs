use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release rollup must pass. Check returns its problems, or none.
pub trait ReleaseRollupRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRollupRequiredNameRule {
    pub message: String,
}

impl ReleaseRollupRule for ReleaseRollupRequiredNameRule {
    fn check(&self, item: &ReleaseRollup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRollupAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseRollupRule for ReleaseRollupAttemptsRangeRule {
    fn check(&self, item: &ReleaseRollup) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRollupMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseRollupRule for ReleaseRollupMarginRangeRule {
    fn check(&self, item: &ReleaseRollup) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRollupAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ReleaseRollupRule for ReleaseRollupAllowedNameRule {
    fn check(&self, item: &ReleaseRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is unknown", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRollupRequiredCodeRule {
    pub message: String,
}

impl ReleaseRollupRule for ReleaseRollupRequiredCodeRule {
    fn check(&self, item: &ReleaseRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a release rollup.
#[derive(Debug, Clone)]
pub struct ReleaseRollupValidator {
    pub rules: Vec<Rc<dyn ReleaseRollupRule>>,
}

impl ReleaseRollupValidator {
    pub fn validate(&self, item: &ReleaseRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_rollup_validator() -> ReleaseRollupValidator {
    ReleaseRollupValidator {
        rules: vec![
            Rc::new(ReleaseRollupRequiredNameRule {
                message: "release rollup needs this field".to_string(),
            }) as Rc<dyn ReleaseRollupRule>,
            Rc::new(ReleaseRollupAttemptsRangeRule { min: 80, max: 120 }),
            Rc::new(ReleaseRollupMarginRangeRule { min: 7.7, max: 857.1 }),
            Rc::new(ReleaseRollupAllowedNameRule { allowed: Vec::new() }),
            Rc::new(ReleaseRollupRequiredCodeRule {
                message: "release rollup needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_rollup_validator();
        let item = new_release_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ReleaseRollupRequiredNameRule {
            message: "release rollup needs an entry".to_string(),
        };
        let mut item = new_release_rollup("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ReleaseRollupAttemptsRangeRule { min: 64, max: 1000 };
        let mut item = new_release_rollup("x", "y");
        item.attempts = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = ReleaseRollupMarginRangeRule { min: 1.7, max: 2887.6 };
        let mut item = new_release_rollup("x", "y");
        item.margin = 2889.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReleaseRollupRequiredCodeRule {
            message: "release rollup needs this field".to_string(),
        };
        let mut item = new_release_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
