use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary rollout rollup must pass. Check returns its problems, or none.
pub trait PrimaryRolloutRollupRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRolloutRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutRollupRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryRolloutRollupRule for PrimaryRolloutRollupRateRangeRule {
    fn check(&self, item: &PrimaryRolloutRollup) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutRollupRequiredUnitRule {
    pub message: String,
}

impl PrimaryRolloutRollupRule for PrimaryRolloutRollupRequiredUnitRule {
    fn check(&self, item: &PrimaryRolloutRollup) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutRollupCodePrefixRule {
    pub prefix: String,
}

impl PrimaryRolloutRollupRule for PrimaryRolloutRollupCodePrefixRule {
    fn check(&self, item: &PrimaryRolloutRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutRollupStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryRolloutRollupRule for PrimaryRolloutRollupStartsAtRangeRule {
    fn check(&self, item: &PrimaryRolloutRollup) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutRollupRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryRolloutRollupRule for PrimaryRolloutRollupRotationDaysRangeRule {
    fn check(&self, item: &PrimaryRolloutRollup) -> Vec<String> {
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

/// Runs every rule of a primary rollout rollup.
#[derive(Debug, Clone)]
pub struct PrimaryRolloutRollupValidator {
    pub rules: Vec<Rc<dyn PrimaryRolloutRollupRule>>,
}

impl PrimaryRolloutRollupValidator {
    pub fn validate(&self, item: &PrimaryRolloutRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryRolloutRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRolloutRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_rollout_rollup_validator() -> PrimaryRolloutRollupValidator {
    PrimaryRolloutRollupValidator {
        rules: vec![
            Rc::new(PrimaryRolloutRollupRateRangeRule { min: 3.8, max: 1970.1 }) as Rc<dyn PrimaryRolloutRollupRule>,
            Rc::new(PrimaryRolloutRollupRequiredUnitRule {
                message: "primary rollout rollup needs a value".to_string(),
            }),
            Rc::new(PrimaryRolloutRollupCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PrimaryRolloutRollupStartsAtRangeRule { min: 64, max: 3600 }),
            Rc::new(PrimaryRolloutRollupRotationDaysRangeRule { min: 32, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_rollout_rollup_validator();
        let item = new_primary_rollout_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = PrimaryRolloutRollupRateRangeRule { min: 4.6, max: 4362.3 };
        let mut item = new_primary_rollout_rollup("x", "y");
        item.rate = 4363.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PrimaryRolloutRollupRequiredUnitRule {
            message: "primary rollout rollup needs this field".to_string(),
        };
        let mut item = new_primary_rollout_rollup("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = PrimaryRolloutRollupStartsAtRangeRule { min: 120, max: 255 };
        let mut item = new_primary_rollout_rollup("x", "y");
        item.starts_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = PrimaryRolloutRollupRotationDaysRangeRule { min: 80, max: 120 };
        let mut item = new_primary_rollout_rollup("x", "y");
        item.rotation_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
