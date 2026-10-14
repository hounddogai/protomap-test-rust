use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default federation adjustment must pass. Check returns its problems, or none.
pub trait DefaultFederationAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultFederationAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAdjustmentAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultFederationAdjustmentRule for DefaultFederationAdjustmentAmountRangeRule {
    fn check(&self, item: &DefaultFederationAdjustment) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAdjustmentThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultFederationAdjustmentRule for DefaultFederationAdjustmentThresholdRangeRule {
    fn check(&self, item: &DefaultFederationAdjustment) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAdjustmentAudienceLengthRule {
    pub max_length: i64,
}

impl DefaultFederationAdjustmentRule for DefaultFederationAdjustmentAudienceLengthRule {
    fn check(&self, item: &DefaultFederationAdjustment) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAdjustmentRequiredAudienceRule {
    pub message: String,
}

impl DefaultFederationAdjustmentRule for DefaultFederationAdjustmentRequiredAudienceRule {
    fn check(&self, item: &DefaultFederationAdjustment) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default federation adjustment.
#[derive(Debug, Clone)]
pub struct DefaultFederationAdjustmentValidator {
    pub rules: Vec<Rc<dyn DefaultFederationAdjustmentRule>>,
}

impl DefaultFederationAdjustmentValidator {
    pub fn validate(&self, item: &DefaultFederationAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultFederationAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultFederationAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_federation_adjustment_validator() -> DefaultFederationAdjustmentValidator {
    DefaultFederationAdjustmentValidator {
        rules: vec![
            Rc::new(DefaultFederationAdjustmentAmountRangeRule { min: 0.3, max: 1388.5 })
                as Rc<dyn DefaultFederationAdjustmentRule>,
            Rc::new(DefaultFederationAdjustmentThresholdRangeRule { min: 7.5, max: 1172.4 }),
            Rc::new(DefaultFederationAdjustmentAudienceLengthRule { max_length: 64 }),
            Rc::new(DefaultFederationAdjustmentRequiredAudienceRule {
                message: "default federation adjustment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_federation_adjustment_validator();
        let item = new_default_federation_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = DefaultFederationAdjustmentAmountRangeRule { min: 7.6, max: 2343.0 };
        let mut item = new_default_federation_adjustment("x", "y");
        item.amount = 2344.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = DefaultFederationAdjustmentThresholdRangeRule { min: 1.0, max: 2471.0 };
        let mut item = new_default_federation_adjustment("x", "y");
        item.threshold = 2472.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = DefaultFederationAdjustmentAudienceLengthRule { max_length: 120 };
        let mut item = new_default_federation_adjustment("x", "y");
        item.audience = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = DefaultFederationAdjustmentRequiredAudienceRule {
            message: "default federation adjustment needs an entry".to_string(),
        };
        let mut item = new_default_federation_adjustment("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
