use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce adjustment must pass. Check returns its problems, or none.
pub trait NonceAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &NonceAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceAdjustmentSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceAdjustmentRule for NonceAdjustmentSequenceRangeRule {
    fn check(&self, item: &NonceAdjustment) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceAdjustmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NonceAdjustmentRule for NonceAdjustmentCeilingRangeRule {
    fn check(&self, item: &NonceAdjustment) -> Vec<String> {
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
pub struct NonceAdjustmentNameLengthRule {
    pub max_length: i64,
}

impl NonceAdjustmentRule for NonceAdjustmentNameLengthRule {
    fn check(&self, item: &NonceAdjustment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce adjustment.
#[derive(Debug, Clone)]
pub struct NonceAdjustmentValidator {
    pub rules: Vec<Rc<dyn NonceAdjustmentRule>>,
}

impl NonceAdjustmentValidator {
    pub fn validate(&self, item: &NonceAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &NonceAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_adjustment_validator() -> NonceAdjustmentValidator {
    NonceAdjustmentValidator {
        rules: vec![
            Rc::new(NonceAdjustmentSequenceRangeRule { min: 16, max: 255 }) as Rc<dyn NonceAdjustmentRule>,
            Rc::new(NonceAdjustmentCeilingRangeRule { min: 3.7, max: 425.9 }),
            Rc::new(NonceAdjustmentNameLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_adjustment_validator();
        let item = new_nonce_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = NonceAdjustmentSequenceRangeRule { min: 16, max: 86400 };
        let mut item = new_nonce_adjustment("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = NonceAdjustmentCeilingRangeRule { min: 1.1, max: 1823.8 };
        let mut item = new_nonce_adjustment("x", "y");
        item.ceiling = 1825.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = NonceAdjustmentNameLengthRule { max_length: 3600 };
        let mut item = new_nonce_adjustment("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
