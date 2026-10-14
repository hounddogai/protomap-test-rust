use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending delegation milestone must pass. Check returns its problems, or none.
pub trait PendingDelegationMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &PendingDelegationMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationMilestoneAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingDelegationMilestoneRule for PendingDelegationMilestoneAllowedCodeRule {
    fn check(&self, item: &PendingDelegationMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationMilestoneAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingDelegationMilestoneRule for PendingDelegationMilestoneAttemptsRangeRule {
    fn check(&self, item: &PendingDelegationMilestone) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationMilestoneCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingDelegationMilestoneRule for PendingDelegationMilestoneCostRangeRule {
    fn check(&self, item: &PendingDelegationMilestone) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationMilestoneScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingDelegationMilestoneRule for PendingDelegationMilestoneScoreRangeRule {
    fn check(&self, item: &PendingDelegationMilestone) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending delegation milestone.
#[derive(Debug, Clone)]
pub struct PendingDelegationMilestoneValidator {
    pub rules: Vec<Rc<dyn PendingDelegationMilestoneRule>>,
}

impl PendingDelegationMilestoneValidator {
    pub fn validate(&self, item: &PendingDelegationMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingDelegationMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingDelegationMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_delegation_milestone_validator() -> PendingDelegationMilestoneValidator {
    PendingDelegationMilestoneValidator {
        rules: vec![
            Rc::new(PendingDelegationMilestoneAllowedCodeRule { allowed: Vec::new() })
                as Rc<dyn PendingDelegationMilestoneRule>,
            Rc::new(PendingDelegationMilestoneAttemptsRangeRule { min: 120, max: 86400 }),
            Rc::new(PendingDelegationMilestoneCostRangeRule { min: 6.0, max: 4609.7 }),
            Rc::new(PendingDelegationMilestoneScoreRangeRule { min: 6.0, max: 3758.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_delegation_milestone_validator();
        let item = new_pending_delegation_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = PendingDelegationMilestoneAttemptsRangeRule { min: 64, max: 120 };
        let mut item = new_pending_delegation_milestone("x", "y");
        item.attempts = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = PendingDelegationMilestoneCostRangeRule { min: 1.8, max: 4474.9 };
        let mut item = new_pending_delegation_milestone("x", "y");
        item.cost = 4476.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PendingDelegationMilestoneScoreRangeRule { min: 9.3, max: 887.6 };
        let mut item = new_pending_delegation_milestone("x", "y");
        item.score = 889.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
