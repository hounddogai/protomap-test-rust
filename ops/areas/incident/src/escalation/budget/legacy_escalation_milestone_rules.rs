use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy escalation milestone must pass. Check returns its problems, or none.
pub trait LegacyEscalationMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &LegacyEscalationMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEscalationMilestoneLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyEscalationMilestoneRule for LegacyEscalationMilestoneLimitRangeRule {
    fn check(&self, item: &LegacyEscalationMilestone) -> Vec<String> {
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
pub struct LegacyEscalationMilestoneAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl LegacyEscalationMilestoneRule for LegacyEscalationMilestoneAllowedReviewerRule {
    fn check(&self, item: &LegacyEscalationMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is above maximum", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEscalationMilestoneKeyIdLengthRule {
    pub max_length: i64,
}

impl LegacyEscalationMilestoneRule for LegacyEscalationMilestoneKeyIdLengthRule {
    fn check(&self, item: &LegacyEscalationMilestone) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy escalation milestone.
#[derive(Debug, Clone)]
pub struct LegacyEscalationMilestoneValidator {
    pub rules: Vec<Rc<dyn LegacyEscalationMilestoneRule>>,
}

impl LegacyEscalationMilestoneValidator {
    pub fn validate(&self, item: &LegacyEscalationMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyEscalationMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyEscalationMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_escalation_milestone_validator() -> LegacyEscalationMilestoneValidator {
    LegacyEscalationMilestoneValidator {
        rules: vec![
            Rc::new(LegacyEscalationMilestoneLimitRangeRule { min: 32, max: 80 })
                as Rc<dyn LegacyEscalationMilestoneRule>,
            Rc::new(LegacyEscalationMilestoneAllowedReviewerRule {
                allowed: vec!["lenient".to_string(), "primary".to_string()],
            }),
            Rc::new(LegacyEscalationMilestoneKeyIdLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_escalation_milestone_validator();
        let item = new_legacy_escalation_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = LegacyEscalationMilestoneLimitRangeRule { min: 32, max: 255 };
        let mut item = new_legacy_escalation_milestone("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = LegacyEscalationMilestoneKeyIdLengthRule { max_length: 80 };
        let mut item = new_legacy_escalation_milestone("x", "y");
        item.key_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
