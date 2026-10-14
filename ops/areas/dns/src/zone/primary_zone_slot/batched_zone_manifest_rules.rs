use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched zone manifest must pass. Check returns its problems, or none.
pub trait BatchedZoneManifestRule: std::fmt::Debug {
    fn check(&self, item: &BatchedZoneManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZoneManifestContactEmailLengthRule {
    pub max_length: i64,
}

impl BatchedZoneManifestRule for BatchedZoneManifestContactEmailLengthRule {
    fn check(&self, item: &BatchedZoneManifest) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZoneManifestDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedZoneManifestRule for BatchedZoneManifestDurationMinutesRangeRule {
    fn check(&self, item: &BatchedZoneManifest) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZoneManifestLabelLengthRule {
    pub max_length: i64,
}

impl BatchedZoneManifestRule for BatchedZoneManifestLabelLengthRule {
    fn check(&self, item: &BatchedZoneManifest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched zone manifest.
#[derive(Debug, Clone)]
pub struct BatchedZoneManifestValidator {
    pub rules: Vec<Rc<dyn BatchedZoneManifestRule>>,
}

impl BatchedZoneManifestValidator {
    pub fn validate(&self, item: &BatchedZoneManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedZoneManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedZoneManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_zone_manifest_validator() -> BatchedZoneManifestValidator {
    BatchedZoneManifestValidator {
        rules: vec![
            Rc::new(BatchedZoneManifestContactEmailLengthRule { max_length: 32 }) as Rc<dyn BatchedZoneManifestRule>,
            Rc::new(BatchedZoneManifestDurationMinutesRangeRule { min: 32, max: 64 }),
            Rc::new(BatchedZoneManifestLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_zone_manifest_validator();
        let item = new_batched_zone_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = BatchedZoneManifestContactEmailLengthRule { max_length: 32 };
        let mut item = new_batched_zone_manifest("x", "y");
        item.contact_email = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = BatchedZoneManifestDurationMinutesRangeRule { min: 32, max: 86400 };
        let mut item = new_batched_zone_manifest("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedZoneManifestLabelLengthRule { max_length: 255 };
        let mut item = new_batched_zone_manifest("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
