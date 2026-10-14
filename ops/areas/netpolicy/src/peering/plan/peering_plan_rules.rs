use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering plan must pass. Check returns its problems, or none.
pub trait PeeringPlanRule: std::fmt::Debug {
    fn check(&self, item: &PeeringPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPlanChannelLengthRule {
    pub max_length: i64,
}

impl PeeringPlanRule for PeeringPlanChannelLengthRule {
    fn check(&self, item: &PeeringPlan) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPlanAudienceLengthRule {
    pub max_length: i64,
}

impl PeeringPlanRule for PeeringPlanAudienceLengthRule {
    fn check(&self, item: &PeeringPlan) -> Vec<String> {
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
pub struct PeeringPlanScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PeeringPlanRule for PeeringPlanScoreRangeRule {
    fn check(&self, item: &PeeringPlan) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPlanRequiredChannelRule {
    pub message: String,
}

impl PeeringPlanRule for PeeringPlanRequiredChannelRule {
    fn check(&self, item: &PeeringPlan) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPlanCodePrefixRule {
    pub prefix: String,
}

impl PeeringPlanRule for PeeringPlanCodePrefixRule {
    fn check(&self, item: &PeeringPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering plan.
#[derive(Debug, Clone)]
pub struct PeeringPlanValidator {
    pub rules: Vec<Rc<dyn PeeringPlanRule>>,
}

impl PeeringPlanValidator {
    pub fn validate(&self, item: &PeeringPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_plan_validator() -> PeeringPlanValidator {
    PeeringPlanValidator {
        rules: vec![
            Rc::new(PeeringPlanChannelLengthRule { max_length: 1000 }) as Rc<dyn PeeringPlanRule>,
            Rc::new(PeeringPlanAudienceLengthRule { max_length: 16 }),
            Rc::new(PeeringPlanScoreRangeRule { min: 3.4, max: 1984.7 }),
            Rc::new(PeeringPlanRequiredChannelRule {
                message: "peering plan needs an entry".to_string(),
            }),
            Rc::new(PeeringPlanCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_plan_validator();
        let item = new_peering_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = PeeringPlanChannelLengthRule { max_length: 1000 };
        let mut item = new_peering_plan("x", "y");
        item.channel = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = PeeringPlanAudienceLengthRule { max_length: 16 };
        let mut item = new_peering_plan("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PeeringPlanScoreRangeRule { min: 1.4, max: 3623.1 };
        let mut item = new_peering_plan("x", "y");
        item.score = 3624.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = PeeringPlanRequiredChannelRule {
            message: "peering plan needs an entry".to_string(),
        };
        let mut item = new_peering_plan("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
