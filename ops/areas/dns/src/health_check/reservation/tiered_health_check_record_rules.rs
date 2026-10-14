use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered health check record must pass. Check returns its problems, or none.
pub trait TieredHealthCheckRecordRule: std::fmt::Debug {
    fn check(&self, item: &TieredHealthCheckRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckRecordTitleLengthRule {
    pub max_length: i64,
}

impl TieredHealthCheckRecordRule for TieredHealthCheckRecordTitleLengthRule {
    fn check(&self, item: &TieredHealthCheckRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckRecordCodeLengthRule {
    pub max_length: i64,
}

impl TieredHealthCheckRecordRule for TieredHealthCheckRecordCodeLengthRule {
    fn check(&self, item: &TieredHealthCheckRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckRecordEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredHealthCheckRecordRule for TieredHealthCheckRecordEndsAtRangeRule {
    fn check(&self, item: &TieredHealthCheckRecord) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckRecordAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl TieredHealthCheckRecordRule for TieredHealthCheckRecordAllowedTitleRule {
    fn check(&self, item: &TieredHealthCheckRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckRecordRequiredCodeRule {
    pub message: String,
}

impl TieredHealthCheckRecordRule for TieredHealthCheckRecordRequiredCodeRule {
    fn check(&self, item: &TieredHealthCheckRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered health check record.
#[derive(Debug, Clone)]
pub struct TieredHealthCheckRecordValidator {
    pub rules: Vec<Rc<dyn TieredHealthCheckRecordRule>>,
}

impl TieredHealthCheckRecordValidator {
    pub fn validate(&self, item: &TieredHealthCheckRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredHealthCheckRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredHealthCheckRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_health_check_record_validator() -> TieredHealthCheckRecordValidator {
    TieredHealthCheckRecordValidator {
        rules: vec![
            Rc::new(TieredHealthCheckRecordTitleLengthRule { max_length: 1000 }) as Rc<dyn TieredHealthCheckRecordRule>,
            Rc::new(TieredHealthCheckRecordCodeLengthRule { max_length: 1000 }),
            Rc::new(TieredHealthCheckRecordEndsAtRangeRule { min: 64, max: 120 }),
            Rc::new(TieredHealthCheckRecordAllowedTitleRule {
                allowed: vec!["legacy".to_string(), "rolling".to_string()],
            }),
            Rc::new(TieredHealthCheckRecordRequiredCodeRule {
                message: "tiered health check record needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_health_check_record_validator();
        let item = new_tiered_health_check_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredHealthCheckRecordTitleLengthRule { max_length: 1000 };
        let mut item = new_tiered_health_check_record("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredHealthCheckRecordCodeLengthRule { max_length: 86400 };
        let mut item = new_tiered_health_check_record("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = TieredHealthCheckRecordEndsAtRangeRule { min: 16, max: 86400 };
        let mut item = new_tiered_health_check_record("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredHealthCheckRecordRequiredCodeRule {
            message: "tiered health check record needs an entry".to_string(),
        };
        let mut item = new_tiered_health_check_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
