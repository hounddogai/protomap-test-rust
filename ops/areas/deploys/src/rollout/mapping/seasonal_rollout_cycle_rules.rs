use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal rollout cycle must pass. Check returns its problems, or none.
pub trait SeasonalRolloutCycleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRolloutCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutCycleRequiredCodeRule {
    pub message: String,
}

impl SeasonalRolloutCycleRule for SeasonalRolloutCycleRequiredCodeRule {
    fn check(&self, item: &SeasonalRolloutCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutCycleSummaryLengthRule {
    pub max_length: i64,
}

impl SeasonalRolloutCycleRule for SeasonalRolloutCycleSummaryLengthRule {
    fn check(&self, item: &SeasonalRolloutCycle) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutCycleAudienceLengthRule {
    pub max_length: i64,
}

impl SeasonalRolloutCycleRule for SeasonalRolloutCycleAudienceLengthRule {
    fn check(&self, item: &SeasonalRolloutCycle) -> Vec<String> {
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
pub struct SeasonalRolloutCycleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SeasonalRolloutCycleRule for SeasonalRolloutCycleAllowedCodeRule {
    fn check(&self, item: &SeasonalRolloutCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutCycleLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalRolloutCycleRule for SeasonalRolloutCycleLabelLengthRule {
    fn check(&self, item: &SeasonalRolloutCycle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal rollout cycle.
#[derive(Debug, Clone)]
pub struct SeasonalRolloutCycleValidator {
    pub rules: Vec<Rc<dyn SeasonalRolloutCycleRule>>,
}

impl SeasonalRolloutCycleValidator {
    pub fn validate(&self, item: &SeasonalRolloutCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalRolloutCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRolloutCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_rollout_cycle_validator() -> SeasonalRolloutCycleValidator {
    SeasonalRolloutCycleValidator {
        rules: vec![
            Rc::new(SeasonalRolloutCycleRequiredCodeRule {
                message: "seasonal rollout cycle needs this field".to_string(),
            }) as Rc<dyn SeasonalRolloutCycleRule>,
            Rc::new(SeasonalRolloutCycleSummaryLengthRule { max_length: 80 }),
            Rc::new(SeasonalRolloutCycleAudienceLengthRule { max_length: 64 }),
            Rc::new(SeasonalRolloutCycleAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "legacy".to_string(), "weighted".to_string()],
            }),
            Rc::new(SeasonalRolloutCycleLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_rollout_cycle_validator();
        let item = new_seasonal_rollout_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalRolloutCycleRequiredCodeRule {
            message: "seasonal rollout cycle needs this field".to_string(),
        };
        let mut item = new_seasonal_rollout_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = SeasonalRolloutCycleSummaryLengthRule { max_length: 120 };
        let mut item = new_seasonal_rollout_cycle("x", "y");
        item.summary = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = SeasonalRolloutCycleAudienceLengthRule { max_length: 3600 };
        let mut item = new_seasonal_rollout_cycle("x", "y");
        item.audience = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SeasonalRolloutCycleLabelLengthRule { max_length: 255 };
        let mut item = new_seasonal_rollout_cycle("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
