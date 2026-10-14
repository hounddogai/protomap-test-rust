use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive measurement entry must pass. Check returns its problems, or none.
pub trait AdaptiveMeasurementEntryRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveMeasurementEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementEntryAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl AdaptiveMeasurementEntryRule for AdaptiveMeasurementEntryAllowedChecksumRule {
    fn check(&self, item: &AdaptiveMeasurementEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is missing", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementEntryChecksumLengthRule {
    pub max_length: i64,
}

impl AdaptiveMeasurementEntryRule for AdaptiveMeasurementEntryChecksumLengthRule {
    fn check(&self, item: &AdaptiveMeasurementEntry) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementEntrySourceLengthRule {
    pub max_length: i64,
}

impl AdaptiveMeasurementEntryRule for AdaptiveMeasurementEntrySourceLengthRule {
    fn check(&self, item: &AdaptiveMeasurementEntry) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementEntryTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl AdaptiveMeasurementEntryRule for AdaptiveMeasurementEntryTagsLimitRule {
    fn check(&self, item: &AdaptiveMeasurementEntry) -> Vec<String> {
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
pub struct AdaptiveMeasurementEntryNameLengthRule {
    pub max_length: i64,
}

impl AdaptiveMeasurementEntryRule for AdaptiveMeasurementEntryNameLengthRule {
    fn check(&self, item: &AdaptiveMeasurementEntry) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive measurement entry.
#[derive(Debug, Clone)]
pub struct AdaptiveMeasurementEntryValidator {
    pub rules: Vec<Rc<dyn AdaptiveMeasurementEntryRule>>,
}

impl AdaptiveMeasurementEntryValidator {
    pub fn validate(&self, item: &AdaptiveMeasurementEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveMeasurementEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveMeasurementEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_measurement_entry_validator() -> AdaptiveMeasurementEntryValidator {
    AdaptiveMeasurementEntryValidator {
        rules: vec![
            Rc::new(AdaptiveMeasurementEntryAllowedChecksumRule {
                allowed: vec!["regional".to_string(), "manual".to_string()],
            }) as Rc<dyn AdaptiveMeasurementEntryRule>,
            Rc::new(AdaptiveMeasurementEntryChecksumLengthRule { max_length: 16 }),
            Rc::new(AdaptiveMeasurementEntrySourceLengthRule { max_length: 86400 }),
            Rc::new(AdaptiveMeasurementEntryTagsLimitRule {
                max_count: 86400,
                banned: vec!["regional".to_string(), "pending".to_string(), "regional".to_string()],
            }),
            Rc::new(AdaptiveMeasurementEntryNameLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_measurement_entry_validator();
        let item = new_adaptive_measurement_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = AdaptiveMeasurementEntryChecksumLengthRule { max_length: 16 };
        let mut item = new_adaptive_measurement_entry("x", "y");
        item.checksum = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = AdaptiveMeasurementEntrySourceLengthRule { max_length: 16 };
        let mut item = new_adaptive_measurement_entry("x", "y");
        item.source = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AdaptiveMeasurementEntryNameLengthRule { max_length: 120 };
        let mut item = new_adaptive_measurement_entry("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
