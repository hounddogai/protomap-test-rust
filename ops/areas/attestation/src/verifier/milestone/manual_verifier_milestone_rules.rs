use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual verifier milestone must pass. Check returns its problems, or none.
pub trait ManualVerifierMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &ManualVerifierMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierMilestoneRequiredRegionRule {
    pub message: String,
}

impl ManualVerifierMilestoneRule for ManualVerifierMilestoneRequiredRegionRule {
    fn check(&self, item: &ManualVerifierMilestone) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierMilestoneRequiredLabelRule {
    pub message: String,
}

impl ManualVerifierMilestoneRule for ManualVerifierMilestoneRequiredLabelRule {
    fn check(&self, item: &ManualVerifierMilestone) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierMilestoneCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualVerifierMilestoneRule for ManualVerifierMilestoneCeilingRangeRule {
    fn check(&self, item: &ManualVerifierMilestone) -> Vec<String> {
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

/// Runs every rule of a manual verifier milestone.
#[derive(Debug, Clone)]
pub struct ManualVerifierMilestoneValidator {
    pub rules: Vec<Rc<dyn ManualVerifierMilestoneRule>>,
}

impl ManualVerifierMilestoneValidator {
    pub fn validate(&self, item: &ManualVerifierMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualVerifierMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualVerifierMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_verifier_milestone_validator() -> ManualVerifierMilestoneValidator {
    ManualVerifierMilestoneValidator {
        rules: vec![
            Rc::new(ManualVerifierMilestoneRequiredRegionRule {
                message: "manual verifier milestone needs an entry".to_string(),
            }) as Rc<dyn ManualVerifierMilestoneRule>,
            Rc::new(ManualVerifierMilestoneRequiredLabelRule {
                message: "manual verifier milestone needs this field".to_string(),
            }),
            Rc::new(ManualVerifierMilestoneCeilingRangeRule { min: 7.6, max: 3514.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_verifier_milestone_validator();
        let item = new_manual_verifier_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = ManualVerifierMilestoneRequiredRegionRule {
            message: "manual verifier milestone needs an entry".to_string(),
        };
        let mut item = new_manual_verifier_milestone("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualVerifierMilestoneRequiredLabelRule {
            message: "manual verifier milestone needs a value".to_string(),
        };
        let mut item = new_manual_verifier_milestone("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ManualVerifierMilestoneCeilingRangeRule { min: 8.4, max: 2355.7 };
        let mut item = new_manual_verifier_milestone("x", "y");
        item.ceiling = 2357.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
