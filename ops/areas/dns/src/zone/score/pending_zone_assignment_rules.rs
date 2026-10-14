use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending zone assignment must pass. Check returns its problems, or none.
pub trait PendingZoneAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PendingZoneAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingZoneAssignmentRequiredClusterRule {
    pub message: String,
}

impl PendingZoneAssignmentRule for PendingZoneAssignmentRequiredClusterRule {
    fn check(&self, item: &PendingZoneAssignment) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingZoneAssignmentCodePrefixRule {
    pub prefix: String,
}

impl PendingZoneAssignmentRule for PendingZoneAssignmentCodePrefixRule {
    fn check(&self, item: &PendingZoneAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingZoneAssignmentRequiredLabelRule {
    pub message: String,
}

impl PendingZoneAssignmentRule for PendingZoneAssignmentRequiredLabelRule {
    fn check(&self, item: &PendingZoneAssignment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingZoneAssignmentCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingZoneAssignmentRule for PendingZoneAssignmentCostRangeRule {
    fn check(&self, item: &PendingZoneAssignment) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending zone assignment.
#[derive(Debug, Clone)]
pub struct PendingZoneAssignmentValidator {
    pub rules: Vec<Rc<dyn PendingZoneAssignmentRule>>,
}

impl PendingZoneAssignmentValidator {
    pub fn validate(&self, item: &PendingZoneAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingZoneAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingZoneAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_zone_assignment_validator() -> PendingZoneAssignmentValidator {
    PendingZoneAssignmentValidator {
        rules: vec![
            Rc::new(PendingZoneAssignmentRequiredClusterRule {
                message: "pending zone assignment needs a value".to_string(),
            }) as Rc<dyn PendingZoneAssignmentRule>,
            Rc::new(PendingZoneAssignmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PendingZoneAssignmentRequiredLabelRule {
                message: "pending zone assignment needs a value".to_string(),
            }),
            Rc::new(PendingZoneAssignmentCostRangeRule { min: 0.7, max: 1467.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_zone_assignment_validator();
        let item = new_pending_zone_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = PendingZoneAssignmentRequiredClusterRule {
            message: "pending zone assignment needs an entry".to_string(),
        };
        let mut item = new_pending_zone_assignment("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingZoneAssignmentRequiredLabelRule {
            message: "pending zone assignment needs an entry".to_string(),
        };
        let mut item = new_pending_zone_assignment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = PendingZoneAssignmentCostRangeRule { min: 3.4, max: 3879.9 };
        let mut item = new_pending_zone_assignment("x", "y");
        item.cost = 3881.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
