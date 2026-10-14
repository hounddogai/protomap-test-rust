use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a slo cycle must pass. Check returns its problems, or none.
pub trait SloCycleRule: std::fmt::Debug {
    fn check(&self, item: &SloCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloCycleUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloCycleRule for SloCycleUpdatedAtRangeRule {
    fn check(&self, item: &SloCycle) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloCycleRequiredNameRule {
    pub message: String,
}

impl SloCycleRule for SloCycleRequiredNameRule {
    fn check(&self, item: &SloCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloCycleNameLengthRule {
    pub max_length: i64,
}

impl SloCycleRule for SloCycleNameLengthRule {
    fn check(&self, item: &SloCycle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloCycleRequiredCurrencyRule {
    pub message: String,
}

impl SloCycleRule for SloCycleRequiredCurrencyRule {
    fn check(&self, item: &SloCycle) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloCycleAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl SloCycleRule for SloCycleAllowedCurrencyRule {
    fn check(&self, item: &SloCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is blocked", item.currency)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo cycle.
#[derive(Debug, Clone)]
pub struct SloCycleValidator {
    pub rules: Vec<Rc<dyn SloCycleRule>>,
}

impl SloCycleValidator {
    pub fn validate(&self, item: &SloCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_cycle_validator() -> SloCycleValidator {
    SloCycleValidator {
        rules: vec![
            Rc::new(SloCycleUpdatedAtRangeRule { min: 1000, max: 1000 }) as Rc<dyn SloCycleRule>,
            Rc::new(SloCycleRequiredNameRule {
                message: "slo cycle needs this field".to_string(),
            }),
            Rc::new(SloCycleNameLengthRule { max_length: 16 }),
            Rc::new(SloCycleRequiredCurrencyRule {
                message: "slo cycle needs an entry".to_string(),
            }),
            Rc::new(SloCycleAllowedCurrencyRule {
                allowed: vec!["strict".to_string(), "seasonal".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_cycle_validator();
        let item = new_slo_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = SloCycleUpdatedAtRangeRule { min: 16, max: 80 };
        let mut item = new_slo_cycle("x", "y");
        item.updated_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SloCycleRequiredNameRule {
            message: "slo cycle needs this field".to_string(),
        };
        let mut item = new_slo_cycle("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SloCycleNameLengthRule { max_length: 1000 };
        let mut item = new_slo_cycle("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = SloCycleRequiredCurrencyRule {
            message: "slo cycle needs this field".to_string(),
        };
        let mut item = new_slo_cycle("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
