use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled suspension manifest must pass. Check returns its problems, or none.
pub trait ScheduledSuspensionManifestRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSuspensionManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionManifestLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledSuspensionManifestRule for ScheduledSuspensionManifestLabelLengthRule {
    fn check(&self, item: &ScheduledSuspensionManifest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionManifestScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledSuspensionManifestRule for ScheduledSuspensionManifestScoreRangeRule {
    fn check(&self, item: &ScheduledSuspensionManifest) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionManifestAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ScheduledSuspensionManifestRule for ScheduledSuspensionManifestAllowedLabelRule {
    fn check(&self, item: &ScheduledSuspensionManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled suspension manifest.
#[derive(Debug, Clone)]
pub struct ScheduledSuspensionManifestValidator {
    pub rules: Vec<Rc<dyn ScheduledSuspensionManifestRule>>,
}

impl ScheduledSuspensionManifestValidator {
    pub fn validate(&self, item: &ScheduledSuspensionManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledSuspensionManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSuspensionManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_suspension_manifest_validator() -> ScheduledSuspensionManifestValidator {
    ScheduledSuspensionManifestValidator {
        rules: vec![
            Rc::new(ScheduledSuspensionManifestLabelLengthRule { max_length: 16 })
                as Rc<dyn ScheduledSuspensionManifestRule>,
            Rc::new(ScheduledSuspensionManifestScoreRangeRule { min: 4.0, max: 2107.5 }),
            Rc::new(ScheduledSuspensionManifestAllowedLabelRule {
                allowed: vec!["strict".to_string(), "tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_suspension_manifest_validator();
        let item = new_scheduled_suspension_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledSuspensionManifestLabelLengthRule { max_length: 3600 };
        let mut item = new_scheduled_suspension_manifest("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ScheduledSuspensionManifestScoreRangeRule { min: 1.5, max: 2533.7 };
        let mut item = new_scheduled_suspension_manifest("x", "y");
        item.score = 2535.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
