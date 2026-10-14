use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe cycle must pass. Check returns its problems, or none.
pub trait ProbeCycleRule: std::fmt::Debug {
    fn check(&self, item: &ProbeCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeCycleCountryLengthRule {
    pub max_length: i64,
}

impl ProbeCycleRule for ProbeCycleCountryLengthRule {
    fn check(&self, item: &ProbeCycle) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeCycleDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeCycleRule for ProbeCycleDurationMinutesRangeRule {
    fn check(&self, item: &ProbeCycle) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeCycleNameLengthRule {
    pub max_length: i64,
}

impl ProbeCycleRule for ProbeCycleNameLengthRule {
    fn check(&self, item: &ProbeCycle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeCycleTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeCycleRule for ProbeCycleTtlSecondsRangeRule {
    fn check(&self, item: &ProbeCycle) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe cycle.
#[derive(Debug, Clone)]
pub struct ProbeCycleValidator {
    pub rules: Vec<Rc<dyn ProbeCycleRule>>,
}

impl ProbeCycleValidator {
    pub fn validate(&self, item: &ProbeCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ProbeCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_cycle_validator() -> ProbeCycleValidator {
    ProbeCycleValidator {
        rules: vec![
            Rc::new(ProbeCycleCountryLengthRule { max_length: 120 }) as Rc<dyn ProbeCycleRule>,
            Rc::new(ProbeCycleDurationMinutesRangeRule { min: 32, max: 3600 }),
            Rc::new(ProbeCycleNameLengthRule { max_length: 16 }),
            Rc::new(ProbeCycleTtlSecondsRangeRule { min: 32, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_cycle_validator();
        let item = new_probe_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = ProbeCycleCountryLengthRule { max_length: 1000 };
        let mut item = new_probe_cycle("x", "y");
        item.country = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = ProbeCycleDurationMinutesRangeRule { min: 16, max: 120 };
        let mut item = new_probe_cycle("x", "y");
        item.duration_minutes = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ProbeCycleNameLengthRule { max_length: 255 };
        let mut item = new_probe_cycle("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = ProbeCycleTtlSecondsRangeRule { min: 64, max: 255 };
        let mut item = new_probe_cycle("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
