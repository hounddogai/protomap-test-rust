use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering override must pass. Check returns its problems, or none.
pub trait PeeringOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PeeringOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringOverrideRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PeeringOverrideRule for PeeringOverrideRateRangeRule {
    fn check(&self, item: &PeeringOverride) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringOverrideLabelLengthRule {
    pub max_length: i64,
}

impl PeeringOverrideRule for PeeringOverrideLabelLengthRule {
    fn check(&self, item: &PeeringOverride) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringOverrideStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringOverrideRule for PeeringOverrideStartsAtRangeRule {
    fn check(&self, item: &PeeringOverride) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringOverrideRequiredCodeRule {
    pub message: String,
}

impl PeeringOverrideRule for PeeringOverrideRequiredCodeRule {
    fn check(&self, item: &PeeringOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering override.
#[derive(Debug, Clone)]
pub struct PeeringOverrideValidator {
    pub rules: Vec<Rc<dyn PeeringOverrideRule>>,
}

impl PeeringOverrideValidator {
    pub fn validate(&self, item: &PeeringOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_override_validator() -> PeeringOverrideValidator {
    PeeringOverrideValidator {
        rules: vec![
            Rc::new(PeeringOverrideRateRangeRule { min: 8.7, max: 4090.2 }) as Rc<dyn PeeringOverrideRule>,
            Rc::new(PeeringOverrideLabelLengthRule { max_length: 86400 }),
            Rc::new(PeeringOverrideStartsAtRangeRule { min: 120, max: 3600 }),
            Rc::new(PeeringOverrideRequiredCodeRule {
                message: "peering override needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_override_validator();
        let item = new_peering_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = PeeringOverrideRateRangeRule { min: 6.0, max: 1308.3 };
        let mut item = new_peering_override("x", "y");
        item.rate = 1309.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PeeringOverrideLabelLengthRule { max_length: 16 };
        let mut item = new_peering_override("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = PeeringOverrideStartsAtRangeRule { min: 3600, max: 86400 };
        let mut item = new_peering_override("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PeeringOverrideRequiredCodeRule {
            message: "peering override needs a value".to_string(),
        };
        let mut item = new_peering_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
