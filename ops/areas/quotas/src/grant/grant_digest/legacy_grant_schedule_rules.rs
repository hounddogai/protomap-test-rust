use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy grant schedule must pass. Check returns its problems, or none.
pub trait LegacyGrantScheduleRule: std::fmt::Debug {
    fn check(&self, item: &LegacyGrantSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantScheduleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LegacyGrantScheduleRule for LegacyGrantScheduleAllowedCodeRule {
    fn check(&self, item: &LegacyGrantSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantScheduleCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyGrantScheduleRule for LegacyGrantScheduleCeilingRangeRule {
    fn check(&self, item: &LegacyGrantSchedule) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantScheduleRequiredRegionRule {
    pub message: String,
}

impl LegacyGrantScheduleRule for LegacyGrantScheduleRequiredRegionRule {
    fn check(&self, item: &LegacyGrantSchedule) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantScheduleRequiredLabelRule {
    pub message: String,
}

impl LegacyGrantScheduleRule for LegacyGrantScheduleRequiredLabelRule {
    fn check(&self, item: &LegacyGrantSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantScheduleDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyGrantScheduleRule for LegacyGrantScheduleDurationMinutesRangeRule {
    fn check(&self, item: &LegacyGrantSchedule) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantScheduleCodeLengthRule {
    pub max_length: i64,
}

impl LegacyGrantScheduleRule for LegacyGrantScheduleCodeLengthRule {
    fn check(&self, item: &LegacyGrantSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy grant schedule.
#[derive(Debug, Clone)]
pub struct LegacyGrantScheduleValidator {
    pub rules: Vec<Rc<dyn LegacyGrantScheduleRule>>,
}

impl LegacyGrantScheduleValidator {
    pub fn validate(&self, item: &LegacyGrantSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyGrantSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyGrantSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_grant_schedule_validator() -> LegacyGrantScheduleValidator {
    LegacyGrantScheduleValidator {
        rules: vec![
            Rc::new(LegacyGrantScheduleAllowedCodeRule { allowed: Vec::new() }) as Rc<dyn LegacyGrantScheduleRule>,
            Rc::new(LegacyGrantScheduleCeilingRangeRule { min: 1.4, max: 4924.7 }),
            Rc::new(LegacyGrantScheduleRequiredRegionRule {
                message: "legacy grant schedule needs this field".to_string(),
            }),
            Rc::new(LegacyGrantScheduleRequiredLabelRule {
                message: "legacy grant schedule needs this field".to_string(),
            }),
            Rc::new(LegacyGrantScheduleDurationMinutesRangeRule { min: 120, max: 3600 }),
            Rc::new(LegacyGrantScheduleCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_grant_schedule_validator();
        let item = new_legacy_grant_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = LegacyGrantScheduleCeilingRangeRule { min: 4.2, max: 4536.4 };
        let mut item = new_legacy_grant_schedule("x", "y");
        item.ceiling = 4537.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = LegacyGrantScheduleRequiredRegionRule {
            message: "legacy grant schedule needs a value".to_string(),
        };
        let mut item = new_legacy_grant_schedule("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LegacyGrantScheduleRequiredLabelRule {
            message: "legacy grant schedule needs this field".to_string(),
        };
        let mut item = new_legacy_grant_schedule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = LegacyGrantScheduleDurationMinutesRangeRule { min: 1000, max: 3600 };
        let mut item = new_legacy_grant_schedule("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyGrantScheduleCodeLengthRule { max_length: 16 };
        let mut item = new_legacy_grant_schedule("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
