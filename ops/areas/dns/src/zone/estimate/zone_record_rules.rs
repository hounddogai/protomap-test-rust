use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone record must pass. Check returns its problems, or none.
pub trait ZoneRecordRule: std::fmt::Debug {
    fn check(&self, item: &ZoneRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRecordRequiredSegmentRule {
    pub message: String,
}

impl ZoneRecordRule for ZoneRecordRequiredSegmentRule {
    fn check(&self, item: &ZoneRecord) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRecordCodeLengthRule {
    pub max_length: i64,
}

impl ZoneRecordRule for ZoneRecordCodeLengthRule {
    fn check(&self, item: &ZoneRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRecordAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ZoneRecordRule for ZoneRecordAllowedNameRule {
    fn check(&self, item: &ZoneRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRecordBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneRecordRule for ZoneRecordBurstRangeRule {
    fn check(&self, item: &ZoneRecord) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRecordNameLengthRule {
    pub max_length: i64,
}

impl ZoneRecordRule for ZoneRecordNameLengthRule {
    fn check(&self, item: &ZoneRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRecordBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ZoneRecordRule for ZoneRecordBudgetRangeRule {
    fn check(&self, item: &ZoneRecord) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone record.
#[derive(Debug, Clone)]
pub struct ZoneRecordValidator {
    pub rules: Vec<Rc<dyn ZoneRecordRule>>,
}

impl ZoneRecordValidator {
    pub fn validate(&self, item: &ZoneRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_record_validator() -> ZoneRecordValidator {
    ZoneRecordValidator {
        rules: vec![
            Rc::new(ZoneRecordRequiredSegmentRule {
                message: "zone record needs a value".to_string(),
            }) as Rc<dyn ZoneRecordRule>,
            Rc::new(ZoneRecordCodeLengthRule { max_length: 64 }),
            Rc::new(ZoneRecordAllowedNameRule {
                allowed: vec!["primary".to_string(), "scheduled".to_string()],
            }),
            Rc::new(ZoneRecordBurstRangeRule { min: 16, max: 86400 }),
            Rc::new(ZoneRecordNameLengthRule { max_length: 255 }),
            Rc::new(ZoneRecordBudgetRangeRule { min: 5.8, max: 2932.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_record_validator();
        let item = new_zone_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = ZoneRecordRequiredSegmentRule {
            message: "zone record needs a value".to_string(),
        };
        let mut item = new_zone_record("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ZoneRecordCodeLengthRule { max_length: 64 };
        let mut item = new_zone_record("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = ZoneRecordBurstRangeRule { min: 255, max: 3600 };
        let mut item = new_zone_record("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ZoneRecordNameLengthRule { max_length: 3600 };
        let mut item = new_zone_record("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = ZoneRecordBudgetRangeRule { min: 0.0, max: 1661.1 };
        let mut item = new_zone_record("x", "y");
        item.budget = 1662.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
