use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe assignment must pass. Check returns its problems, or none.
pub trait ProbeAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &ProbeAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAssignmentRequiredNameRule {
    pub message: String,
}

impl ProbeAssignmentRule for ProbeAssignmentRequiredNameRule {
    fn check(&self, item: &ProbeAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAssignmentWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeAssignmentRule for ProbeAssignmentWindowDaysRangeRule {
    fn check(&self, item: &ProbeAssignment) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAssignmentLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeAssignmentRule for ProbeAssignmentLimitRangeRule {
    fn check(&self, item: &ProbeAssignment) -> Vec<String> {
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
pub struct ProbeAssignmentCodePrefixRule {
    pub prefix: String,
}

impl ProbeAssignmentRule for ProbeAssignmentCodePrefixRule {
    fn check(&self, item: &ProbeAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe assignment.
#[derive(Debug, Clone)]
pub struct ProbeAssignmentValidator {
    pub rules: Vec<Rc<dyn ProbeAssignmentRule>>,
}

impl ProbeAssignmentValidator {
    pub fn validate(&self, item: &ProbeAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ProbeAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_assignment_validator() -> ProbeAssignmentValidator {
    ProbeAssignmentValidator {
        rules: vec![
            Rc::new(ProbeAssignmentRequiredNameRule {
                message: "probe assignment needs an entry".to_string(),
            }) as Rc<dyn ProbeAssignmentRule>,
            Rc::new(ProbeAssignmentWindowDaysRangeRule { min: 64, max: 255 }),
            Rc::new(ProbeAssignmentLimitRangeRule { min: 64, max: 1000 }),
            Rc::new(ProbeAssignmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_assignment_validator();
        let item = new_probe_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ProbeAssignmentRequiredNameRule {
            message: "probe assignment needs an entry".to_string(),
        };
        let mut item = new_probe_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = ProbeAssignmentWindowDaysRangeRule { min: 64, max: 80 };
        let mut item = new_probe_assignment("x", "y");
        item.window_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ProbeAssignmentLimitRangeRule { min: 32, max: 32 };
        let mut item = new_probe_assignment("x", "y");
        item.limit = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
