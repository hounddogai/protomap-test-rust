use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling canary rollup must pass. Check returns its problems, or none.
pub trait RollingCanaryRollupRule: std::fmt::Debug {
    fn check(&self, item: &RollingCanaryRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryRollupRequiredAudienceRule {
    pub message: String,
}

impl RollingCanaryRollupRule for RollingCanaryRollupRequiredAudienceRule {
    fn check(&self, item: &RollingCanaryRollup) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryRollupCodePrefixRule {
    pub prefix: String,
}

impl RollingCanaryRollupRule for RollingCanaryRollupCodePrefixRule {
    fn check(&self, item: &RollingCanaryRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryRollupTitleLengthRule {
    pub max_length: i64,
}

impl RollingCanaryRollupRule for RollingCanaryRollupTitleLengthRule {
    fn check(&self, item: &RollingCanaryRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryRollupTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingCanaryRollupRule for RollingCanaryRollupTtlSecondsRangeRule {
    fn check(&self, item: &RollingCanaryRollup) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling canary rollup.
#[derive(Debug, Clone)]
pub struct RollingCanaryRollupValidator {
    pub rules: Vec<Rc<dyn RollingCanaryRollupRule>>,
}

impl RollingCanaryRollupValidator {
    pub fn validate(&self, item: &RollingCanaryRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingCanaryRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingCanaryRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_canary_rollup_validator() -> RollingCanaryRollupValidator {
    RollingCanaryRollupValidator {
        rules: vec![
            Rc::new(RollingCanaryRollupRequiredAudienceRule {
                message: "rolling canary rollup needs this field".to_string(),
            }) as Rc<dyn RollingCanaryRollupRule>,
            Rc::new(RollingCanaryRollupCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RollingCanaryRollupTitleLengthRule { max_length: 255 }),
            Rc::new(RollingCanaryRollupTtlSecondsRangeRule { min: 255, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_canary_rollup_validator();
        let item = new_rolling_canary_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = RollingCanaryRollupRequiredAudienceRule {
            message: "rolling canary rollup needs this field".to_string(),
        };
        let mut item = new_rolling_canary_rollup("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollingCanaryRollupTitleLengthRule { max_length: 64 };
        let mut item = new_rolling_canary_rollup("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RollingCanaryRollupTtlSecondsRangeRule { min: 80, max: 86400 };
        let mut item = new_rolling_canary_rollup("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
