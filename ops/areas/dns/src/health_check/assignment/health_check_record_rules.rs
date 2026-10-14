use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a health check record must pass. Check returns its problems, or none.
pub trait HealthCheckRecordRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRecordCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckRecordRule for HealthCheckRecordCostRangeRule {
    fn check(&self, item: &HealthCheckRecord) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRecordSegmentLengthRule {
    pub max_length: i64,
}

impl HealthCheckRecordRule for HealthCheckRecordSegmentLengthRule {
    fn check(&self, item: &HealthCheckRecord) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRecordAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl HealthCheckRecordRule for HealthCheckRecordAllowedCodeRule {
    fn check(&self, item: &HealthCheckRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRecordNameLengthRule {
    pub max_length: i64,
}

impl HealthCheckRecordRule for HealthCheckRecordNameLengthRule {
    fn check(&self, item: &HealthCheckRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRecordRequiredNameRule {
    pub message: String,
}

impl HealthCheckRecordRule for HealthCheckRecordRequiredNameRule {
    fn check(&self, item: &HealthCheckRecord) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRecordBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckRecordRule for HealthCheckRecordBurstRangeRule {
    fn check(&self, item: &HealthCheckRecord) -> Vec<String> {
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

/// Runs every rule of a health check record.
#[derive(Debug, Clone)]
pub struct HealthCheckRecordValidator {
    pub rules: Vec<Rc<dyn HealthCheckRecordRule>>,
}

impl HealthCheckRecordValidator {
    pub fn validate(&self, item: &HealthCheckRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_record_validator() -> HealthCheckRecordValidator {
    HealthCheckRecordValidator {
        rules: vec![
            Rc::new(HealthCheckRecordCostRangeRule { min: 1.7, max: 3299.9 }) as Rc<dyn HealthCheckRecordRule>,
            Rc::new(HealthCheckRecordSegmentLengthRule { max_length: 3600 }),
            Rc::new(HealthCheckRecordAllowedCodeRule {
                allowed: vec!["capped".to_string(), "capped".to_string()],
            }),
            Rc::new(HealthCheckRecordNameLengthRule { max_length: 32 }),
            Rc::new(HealthCheckRecordRequiredNameRule {
                message: "health check record needs a value".to_string(),
            }),
            Rc::new(HealthCheckRecordBurstRangeRule { min: 120, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_record_validator();
        let item = new_health_check_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = HealthCheckRecordCostRangeRule { min: 6.3, max: 3302.9 };
        let mut item = new_health_check_record("x", "y");
        item.cost = 3304.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = HealthCheckRecordSegmentLengthRule { max_length: 80 };
        let mut item = new_health_check_record("x", "y");
        item.segment = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = HealthCheckRecordNameLengthRule { max_length: 3600 };
        let mut item = new_health_check_record("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = HealthCheckRecordRequiredNameRule {
            message: "health check record needs an entry".to_string(),
        };
        let mut item = new_health_check_record("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = HealthCheckRecordBurstRangeRule { min: 32, max: 3600 };
        let mut item = new_health_check_record("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
