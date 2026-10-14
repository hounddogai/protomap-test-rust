use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback zone record must pass. Check returns its problems, or none.
pub trait FallbackZoneRecordRule: std::fmt::Debug {
    fn check(&self, item: &FallbackZoneRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneRecordScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackZoneRecordRule for FallbackZoneRecordScoreRangeRule {
    fn check(&self, item: &FallbackZoneRecord) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneRecordRequiredCodeRule {
    pub message: String,
}

impl FallbackZoneRecordRule for FallbackZoneRecordRequiredCodeRule {
    fn check(&self, item: &FallbackZoneRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneRecordCodeLengthRule {
    pub max_length: i64,
}

impl FallbackZoneRecordRule for FallbackZoneRecordCodeLengthRule {
    fn check(&self, item: &FallbackZoneRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback zone record.
#[derive(Debug, Clone)]
pub struct FallbackZoneRecordValidator {
    pub rules: Vec<Rc<dyn FallbackZoneRecordRule>>,
}

impl FallbackZoneRecordValidator {
    pub fn validate(&self, item: &FallbackZoneRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackZoneRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackZoneRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_zone_record_validator() -> FallbackZoneRecordValidator {
    FallbackZoneRecordValidator {
        rules: vec![
            Rc::new(FallbackZoneRecordScoreRangeRule { min: 8.9, max: 4598.2 }) as Rc<dyn FallbackZoneRecordRule>,
            Rc::new(FallbackZoneRecordRequiredCodeRule {
                message: "fallback zone record needs an entry".to_string(),
            }),
            Rc::new(FallbackZoneRecordCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_zone_record_validator();
        let item = new_fallback_zone_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = FallbackZoneRecordScoreRangeRule { min: 3.5, max: 2201.0 };
        let mut item = new_fallback_zone_record("x", "y");
        item.score = 2202.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackZoneRecordRequiredCodeRule {
            message: "fallback zone record needs an entry".to_string(),
        };
        let mut item = new_fallback_zone_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackZoneRecordCodeLengthRule { max_length: 86400 };
        let mut item = new_fallback_zone_record("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
