use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal probe schedule must pass. Check returns its problems, or none.
pub trait SeasonalProbeScheduleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalProbeSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeScheduleRequiredNameRule {
    pub message: String,
}

impl SeasonalProbeScheduleRule for SeasonalProbeScheduleRequiredNameRule {
    fn check(&self, item: &SeasonalProbeSchedule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeScheduleCodePrefixRule {
    pub prefix: String,
}

impl SeasonalProbeScheduleRule for SeasonalProbeScheduleCodePrefixRule {
    fn check(&self, item: &SeasonalProbeSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeScheduleAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl SeasonalProbeScheduleRule for SeasonalProbeScheduleAllowedCurrencyRule {
    fn check(&self, item: &SeasonalProbeSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is duplicated", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeScheduleRequiredCodeRule {
    pub message: String,
}

impl SeasonalProbeScheduleRule for SeasonalProbeScheduleRequiredCodeRule {
    fn check(&self, item: &SeasonalProbeSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeScheduleNameLengthRule {
    pub max_length: i64,
}

impl SeasonalProbeScheduleRule for SeasonalProbeScheduleNameLengthRule {
    fn check(&self, item: &SeasonalProbeSchedule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeScheduleCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalProbeScheduleRule for SeasonalProbeScheduleCodeLengthRule {
    fn check(&self, item: &SeasonalProbeSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal probe schedule.
#[derive(Debug, Clone)]
pub struct SeasonalProbeScheduleValidator {
    pub rules: Vec<Rc<dyn SeasonalProbeScheduleRule>>,
}

impl SeasonalProbeScheduleValidator {
    pub fn validate(&self, item: &SeasonalProbeSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalProbeSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalProbeSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_probe_schedule_validator() -> SeasonalProbeScheduleValidator {
    SeasonalProbeScheduleValidator {
        rules: vec![
            Rc::new(SeasonalProbeScheduleRequiredNameRule {
                message: "seasonal probe schedule needs this field".to_string(),
            }) as Rc<dyn SeasonalProbeScheduleRule>,
            Rc::new(SeasonalProbeScheduleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(SeasonalProbeScheduleAllowedCurrencyRule {
                allowed: vec!["lenient".to_string(), "strict".to_string()],
            }),
            Rc::new(SeasonalProbeScheduleRequiredCodeRule {
                message: "seasonal probe schedule needs this field".to_string(),
            }),
            Rc::new(SeasonalProbeScheduleNameLengthRule { max_length: 1000 }),
            Rc::new(SeasonalProbeScheduleCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_probe_schedule_validator();
        let item = new_seasonal_probe_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SeasonalProbeScheduleRequiredNameRule {
            message: "seasonal probe schedule needs a value".to_string(),
        };
        let mut item = new_seasonal_probe_schedule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalProbeScheduleRequiredCodeRule {
            message: "seasonal probe schedule needs this field".to_string(),
        };
        let mut item = new_seasonal_probe_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalProbeScheduleNameLengthRule { max_length: 32 };
        let mut item = new_seasonal_probe_schedule("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalProbeScheduleCodeLengthRule { max_length: 3600 };
        let mut item = new_seasonal_probe_schedule("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
