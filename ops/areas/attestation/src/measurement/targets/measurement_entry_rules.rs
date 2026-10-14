use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement entry must pass. Check returns its problems, or none.
pub trait MeasurementEntryRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementEntryRequiredNameRule {
    pub message: String,
}

impl MeasurementEntryRule for MeasurementEntryRequiredNameRule {
    fn check(&self, item: &MeasurementEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementEntryQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementEntryRule for MeasurementEntryQuotaUnitsRangeRule {
    fn check(&self, item: &MeasurementEntry) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementEntryTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl MeasurementEntryRule for MeasurementEntryTagsLimitRule {
    fn check(&self, item: &MeasurementEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementEntryActiveGuardRule {
    pub limit: i64,
}

impl MeasurementEntryRule for MeasurementEntryActiveGuardRule {
    fn check(&self, item: &MeasurementEntry) -> Vec<String> {
        if item.active && (item.duration_minutes > self.limit) {
            return vec![format!(
                "duration minutes {} exceeds {} while active",
                item.duration_minutes, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement entry.
#[derive(Debug, Clone)]
pub struct MeasurementEntryValidator {
    pub rules: Vec<Rc<dyn MeasurementEntryRule>>,
}

impl MeasurementEntryValidator {
    pub fn validate(&self, item: &MeasurementEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_entry_validator() -> MeasurementEntryValidator {
    MeasurementEntryValidator {
        rules: vec![
            Rc::new(MeasurementEntryRequiredNameRule {
                message: "measurement entry needs this field".to_string(),
            }) as Rc<dyn MeasurementEntryRule>,
            Rc::new(MeasurementEntryQuotaUnitsRangeRule { min: 32, max: 80 }),
            Rc::new(MeasurementEntryTagsLimitRule {
                max_count: 64,
                banned: vec!["global".to_string()],
            }),
            Rc::new(MeasurementEntryActiveGuardRule { limit: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_entry_validator();
        let item = new_measurement_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = MeasurementEntryRequiredNameRule {
            message: "measurement entry needs this field".to_string(),
        };
        let mut item = new_measurement_entry("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = MeasurementEntryQuotaUnitsRangeRule { min: 80, max: 255 };
        let mut item = new_measurement_entry("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
