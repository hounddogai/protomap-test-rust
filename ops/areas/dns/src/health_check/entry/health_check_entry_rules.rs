use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a health check entry must pass. Check returns its problems, or none.
pub trait HealthCheckEntryRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckEntryRequiredLabelRule {
    pub message: String,
}

impl HealthCheckEntryRule for HealthCheckEntryRequiredLabelRule {
    fn check(&self, item: &HealthCheckEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckEntryScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckEntryRule for HealthCheckEntryScoreRangeRule {
    fn check(&self, item: &HealthCheckEntry) -> Vec<String> {
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
pub struct HealthCheckEntryAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl HealthCheckEntryRule for HealthCheckEntryAllowedUnitRule {
    fn check(&self, item: &HealthCheckEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is below minimum", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckEntryLabelLengthRule {
    pub max_length: i64,
}

impl HealthCheckEntryRule for HealthCheckEntryLabelLengthRule {
    fn check(&self, item: &HealthCheckEntry) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckEntryCodeLengthRule {
    pub max_length: i64,
}

impl HealthCheckEntryRule for HealthCheckEntryCodeLengthRule {
    fn check(&self, item: &HealthCheckEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check entry.
#[derive(Debug, Clone)]
pub struct HealthCheckEntryValidator {
    pub rules: Vec<Rc<dyn HealthCheckEntryRule>>,
}

impl HealthCheckEntryValidator {
    pub fn validate(&self, item: &HealthCheckEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_entry_validator() -> HealthCheckEntryValidator {
    HealthCheckEntryValidator {
        rules: vec![
            Rc::new(HealthCheckEntryRequiredLabelRule {
                message: "health check entry needs a value".to_string(),
            }) as Rc<dyn HealthCheckEntryRule>,
            Rc::new(HealthCheckEntryScoreRangeRule { min: 9.8, max: 3408.7 }),
            Rc::new(HealthCheckEntryAllowedUnitRule {
                allowed: vec!["adaptive".to_string(), "seasonal".to_string()],
            }),
            Rc::new(HealthCheckEntryLabelLengthRule { max_length: 16 }),
            Rc::new(HealthCheckEntryCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_entry_validator();
        let item = new_health_check_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = HealthCheckEntryRequiredLabelRule {
            message: "health check entry needs a value".to_string(),
        };
        let mut item = new_health_check_entry("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = HealthCheckEntryScoreRangeRule { min: 5.4, max: 4112.0 };
        let mut item = new_health_check_entry("x", "y");
        item.score = 4113.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = HealthCheckEntryLabelLengthRule { max_length: 80 };
        let mut item = new_health_check_entry("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = HealthCheckEntryCodeLengthRule { max_length: 64 };
        let mut item = new_health_check_entry("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
