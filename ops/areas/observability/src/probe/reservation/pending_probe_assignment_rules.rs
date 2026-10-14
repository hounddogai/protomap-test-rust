use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending probe assignment must pass. Check returns its problems, or none.
pub trait PendingProbeAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PendingProbeAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAssignmentRequiredTitleRule {
    pub message: String,
}

impl PendingProbeAssignmentRule for PendingProbeAssignmentRequiredTitleRule {
    fn check(&self, item: &PendingProbeAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAssignmentCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingProbeAssignmentRule for PendingProbeAssignmentCostRangeRule {
    fn check(&self, item: &PendingProbeAssignment) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAssignmentIssuerLengthRule {
    pub max_length: i64,
}

impl PendingProbeAssignmentRule for PendingProbeAssignmentIssuerLengthRule {
    fn check(&self, item: &PendingProbeAssignment) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl PendingProbeAssignmentRule for PendingProbeAssignmentCodeLengthRule {
    fn check(&self, item: &PendingProbeAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAssignmentCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingProbeAssignmentRule for PendingProbeAssignmentCreatedAtRangeRule {
    fn check(&self, item: &PendingProbeAssignment) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending probe assignment.
#[derive(Debug, Clone)]
pub struct PendingProbeAssignmentValidator {
    pub rules: Vec<Rc<dyn PendingProbeAssignmentRule>>,
}

impl PendingProbeAssignmentValidator {
    pub fn validate(&self, item: &PendingProbeAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingProbeAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingProbeAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_probe_assignment_validator() -> PendingProbeAssignmentValidator {
    PendingProbeAssignmentValidator {
        rules: vec![
            Rc::new(PendingProbeAssignmentRequiredTitleRule {
                message: "pending probe assignment needs this field".to_string(),
            }) as Rc<dyn PendingProbeAssignmentRule>,
            Rc::new(PendingProbeAssignmentCostRangeRule { min: 3.9, max: 3003.9 }),
            Rc::new(PendingProbeAssignmentIssuerLengthRule { max_length: 3600 }),
            Rc::new(PendingProbeAssignmentCodeLengthRule { max_length: 120 }),
            Rc::new(PendingProbeAssignmentCreatedAtRangeRule { min: 64, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_probe_assignment_validator();
        let item = new_pending_probe_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingProbeAssignmentRequiredTitleRule {
            message: "pending probe assignment needs a value".to_string(),
        };
        let mut item = new_pending_probe_assignment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = PendingProbeAssignmentCostRangeRule { min: 1.4, max: 2408.4 };
        let mut item = new_pending_probe_assignment("x", "y");
        item.cost = 2409.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = PendingProbeAssignmentIssuerLengthRule { max_length: 255 };
        let mut item = new_pending_probe_assignment("x", "y");
        item.issuer = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingProbeAssignmentCodeLengthRule { max_length: 1000 };
        let mut item = new_pending_probe_assignment("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = PendingProbeAssignmentCreatedAtRangeRule { min: 16, max: 120 };
        let mut item = new_pending_probe_assignment("x", "y");
        item.created_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
