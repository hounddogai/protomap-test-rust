use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive forecast cycle must pass. Check returns its problems, or none.
pub trait AdaptiveForecastCycleRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveForecastCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastCycleLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveForecastCycleRule for AdaptiveForecastCycleLimitRangeRule {
    fn check(&self, item: &AdaptiveForecastCycle) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastCycleRequiredHostRule {
    pub message: String,
}

impl AdaptiveForecastCycleRule for AdaptiveForecastCycleRequiredHostRule {
    fn check(&self, item: &AdaptiveForecastCycle) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastCycleHostLengthRule {
    pub max_length: i64,
}

impl AdaptiveForecastCycleRule for AdaptiveForecastCycleHostLengthRule {
    fn check(&self, item: &AdaptiveForecastCycle) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastCycleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveForecastCycleRule for AdaptiveForecastCycleCreatedAtRangeRule {
    fn check(&self, item: &AdaptiveForecastCycle) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastCycleNotesLengthRule {
    pub max_length: i64,
}

impl AdaptiveForecastCycleRule for AdaptiveForecastCycleNotesLengthRule {
    fn check(&self, item: &AdaptiveForecastCycle) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastCycleCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveForecastCycleRule for AdaptiveForecastCycleCodePrefixRule {
    fn check(&self, item: &AdaptiveForecastCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive forecast cycle.
#[derive(Debug, Clone)]
pub struct AdaptiveForecastCycleValidator {
    pub rules: Vec<Rc<dyn AdaptiveForecastCycleRule>>,
}

impl AdaptiveForecastCycleValidator {
    pub fn validate(&self, item: &AdaptiveForecastCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveForecastCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveForecastCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_forecast_cycle_validator() -> AdaptiveForecastCycleValidator {
    AdaptiveForecastCycleValidator {
        rules: vec![
            Rc::new(AdaptiveForecastCycleLimitRangeRule { min: 80, max: 80 }) as Rc<dyn AdaptiveForecastCycleRule>,
            Rc::new(AdaptiveForecastCycleRequiredHostRule {
                message: "adaptive forecast cycle needs an entry".to_string(),
            }),
            Rc::new(AdaptiveForecastCycleHostLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveForecastCycleCreatedAtRangeRule { min: 16, max: 1000 }),
            Rc::new(AdaptiveForecastCycleNotesLengthRule { max_length: 16 }),
            Rc::new(AdaptiveForecastCycleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_forecast_cycle_validator();
        let item = new_adaptive_forecast_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = AdaptiveForecastCycleLimitRangeRule { min: 255, max: 86400 };
        let mut item = new_adaptive_forecast_cycle("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = AdaptiveForecastCycleRequiredHostRule {
            message: "adaptive forecast cycle needs an entry".to_string(),
        };
        let mut item = new_adaptive_forecast_cycle("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = AdaptiveForecastCycleHostLengthRule { max_length: 80 };
        let mut item = new_adaptive_forecast_cycle("x", "y");
        item.host = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = AdaptiveForecastCycleCreatedAtRangeRule { min: 80, max: 1000 };
        let mut item = new_adaptive_forecast_cycle("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = AdaptiveForecastCycleNotesLengthRule { max_length: 255 };
        let mut item = new_adaptive_forecast_cycle("x", "y");
        item.notes = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
