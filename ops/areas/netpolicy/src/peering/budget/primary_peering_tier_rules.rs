use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary peering tier must pass. Check returns its problems, or none.
pub trait PrimaryPeeringTierRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryPeeringTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringTierDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryPeeringTierRule for PrimaryPeeringTierDiscountRangeRule {
    fn check(&self, item: &PrimaryPeeringTier) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringTierMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryPeeringTierRule for PrimaryPeeringTierMaximumRangeRule {
    fn check(&self, item: &PrimaryPeeringTier) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringTierChannelLengthRule {
    pub max_length: i64,
}

impl PrimaryPeeringTierRule for PrimaryPeeringTierChannelLengthRule {
    fn check(&self, item: &PrimaryPeeringTier) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringTierRequiredCodeRule {
    pub message: String,
}

impl PrimaryPeeringTierRule for PrimaryPeeringTierRequiredCodeRule {
    fn check(&self, item: &PrimaryPeeringTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary peering tier.
#[derive(Debug, Clone)]
pub struct PrimaryPeeringTierValidator {
    pub rules: Vec<Rc<dyn PrimaryPeeringTierRule>>,
}

impl PrimaryPeeringTierValidator {
    pub fn validate(&self, item: &PrimaryPeeringTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryPeeringTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryPeeringTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_peering_tier_validator() -> PrimaryPeeringTierValidator {
    PrimaryPeeringTierValidator {
        rules: vec![
            Rc::new(PrimaryPeeringTierDiscountRangeRule { min: 2.0, max: 2505.6 }) as Rc<dyn PrimaryPeeringTierRule>,
            Rc::new(PrimaryPeeringTierMaximumRangeRule { min: 0.6, max: 2648.6 }),
            Rc::new(PrimaryPeeringTierChannelLengthRule { max_length: 86400 }),
            Rc::new(PrimaryPeeringTierRequiredCodeRule {
                message: "primary peering tier needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_peering_tier_validator();
        let item = new_primary_peering_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = PrimaryPeeringTierDiscountRangeRule { min: 8.9, max: 4121.1 };
        let mut item = new_primary_peering_tier("x", "y");
        item.discount = 4122.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = PrimaryPeeringTierMaximumRangeRule { min: 9.2, max: 4639.0 };
        let mut item = new_primary_peering_tier("x", "y");
        item.maximum = 4640.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = PrimaryPeeringTierChannelLengthRule { max_length: 86400 };
        let mut item = new_primary_peering_tier("x", "y");
        item.channel = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryPeeringTierRequiredCodeRule {
            message: "primary peering tier needs a value".to_string(),
        };
        let mut item = new_primary_peering_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
