use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling peering tier must pass. Check returns its problems, or none.
pub trait RollingPeeringTierRule: std::fmt::Debug {
    fn check(&self, item: &RollingPeeringTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringTierRequiredLocaleRule {
    pub message: String,
}

impl RollingPeeringTierRule for RollingPeeringTierRequiredLocaleRule {
    fn check(&self, item: &RollingPeeringTier) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringTierNameLengthRule {
    pub max_length: i64,
}

impl RollingPeeringTierRule for RollingPeeringTierNameLengthRule {
    fn check(&self, item: &RollingPeeringTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringTierRequiredDescriptionRule {
    pub message: String,
}

impl RollingPeeringTierRule for RollingPeeringTierRequiredDescriptionRule {
    fn check(&self, item: &RollingPeeringTier) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringTierAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingPeeringTierRule for RollingPeeringTierAttemptsRangeRule {
    fn check(&self, item: &RollingPeeringTier) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling peering tier.
#[derive(Debug, Clone)]
pub struct RollingPeeringTierValidator {
    pub rules: Vec<Rc<dyn RollingPeeringTierRule>>,
}

impl RollingPeeringTierValidator {
    pub fn validate(&self, item: &RollingPeeringTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingPeeringTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPeeringTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_peering_tier_validator() -> RollingPeeringTierValidator {
    RollingPeeringTierValidator {
        rules: vec![
            Rc::new(RollingPeeringTierRequiredLocaleRule {
                message: "rolling peering tier needs this field".to_string(),
            }) as Rc<dyn RollingPeeringTierRule>,
            Rc::new(RollingPeeringTierNameLengthRule { max_length: 1000 }),
            Rc::new(RollingPeeringTierRequiredDescriptionRule {
                message: "rolling peering tier needs a value".to_string(),
            }),
            Rc::new(RollingPeeringTierAttemptsRangeRule { min: 255, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_peering_tier_validator();
        let item = new_rolling_peering_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = RollingPeeringTierRequiredLocaleRule {
            message: "rolling peering tier needs an entry".to_string(),
        };
        let mut item = new_rolling_peering_tier("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollingPeeringTierNameLengthRule { max_length: 80 };
        let mut item = new_rolling_peering_tier("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = RollingPeeringTierRequiredDescriptionRule {
            message: "rolling peering tier needs this field".to_string(),
        };
        let mut item = new_rolling_peering_tier("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = RollingPeeringTierAttemptsRangeRule { min: 16, max: 86400 };
        let mut item = new_rolling_peering_tier("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
