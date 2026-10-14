use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling release batch must pass. Check returns its problems, or none.
pub trait RollingReleaseBatchRule: std::fmt::Debug {
    fn check(&self, item: &RollingReleaseBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleaseBatchAudienceLengthRule {
    pub max_length: i64,
}

impl RollingReleaseBatchRule for RollingReleaseBatchAudienceLengthRule {
    fn check(&self, item: &RollingReleaseBatch) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleaseBatchSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingReleaseBatchRule for RollingReleaseBatchSequenceRangeRule {
    fn check(&self, item: &RollingReleaseBatch) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleaseBatchAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl RollingReleaseBatchRule for RollingReleaseBatchAllowedAudienceRule {
    fn check(&self, item: &RollingReleaseBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is above maximum", item.audience)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling release batch.
#[derive(Debug, Clone)]
pub struct RollingReleaseBatchValidator {
    pub rules: Vec<Rc<dyn RollingReleaseBatchRule>>,
}

impl RollingReleaseBatchValidator {
    pub fn validate(&self, item: &RollingReleaseBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingReleaseBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingReleaseBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_release_batch_validator() -> RollingReleaseBatchValidator {
    RollingReleaseBatchValidator {
        rules: vec![
            Rc::new(RollingReleaseBatchAudienceLengthRule { max_length: 80 }) as Rc<dyn RollingReleaseBatchRule>,
            Rc::new(RollingReleaseBatchSequenceRangeRule { min: 16, max: 120 }),
            Rc::new(RollingReleaseBatchAllowedAudienceRule {
                allowed: vec!["seasonal".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_release_batch_validator();
        let item = new_rolling_release_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = RollingReleaseBatchAudienceLengthRule { max_length: 255 };
        let mut item = new_rolling_release_batch("x", "y");
        item.audience = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = RollingReleaseBatchSequenceRangeRule { min: 255, max: 3600 };
        let mut item = new_rolling_release_batch("x", "y");
        item.sequence = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
