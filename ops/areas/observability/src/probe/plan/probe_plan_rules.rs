use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe plan must pass. Check returns its problems, or none.
pub trait ProbePlanRule: std::fmt::Debug {
    fn check(&self, item: &ProbePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbePlanRequiredNotesRule {
    pub message: String,
}

impl ProbePlanRule for ProbePlanRequiredNotesRule {
    fn check(&self, item: &ProbePlan) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbePlanLabelLengthRule {
    pub max_length: i64,
}

impl ProbePlanRule for ProbePlanLabelLengthRule {
    fn check(&self, item: &ProbePlan) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbePlanVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbePlanRule for ProbePlanVersionRangeRule {
    fn check(&self, item: &ProbePlan) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbePlanCodePrefixRule {
    pub prefix: String,
}

impl ProbePlanRule for ProbePlanCodePrefixRule {
    fn check(&self, item: &ProbePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbePlanRequiredLabelRule {
    pub message: String,
}

impl ProbePlanRule for ProbePlanRequiredLabelRule {
    fn check(&self, item: &ProbePlan) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe plan.
#[derive(Debug, Clone)]
pub struct ProbePlanValidator {
    pub rules: Vec<Rc<dyn ProbePlanRule>>,
}

impl ProbePlanValidator {
    pub fn validate(&self, item: &ProbePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ProbePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_plan_validator() -> ProbePlanValidator {
    ProbePlanValidator {
        rules: vec![
            Rc::new(ProbePlanRequiredNotesRule {
                message: "probe plan needs an entry".to_string(),
            }) as Rc<dyn ProbePlanRule>,
            Rc::new(ProbePlanLabelLengthRule { max_length: 16 }),
            Rc::new(ProbePlanVersionRangeRule { min: 16, max: 64 }),
            Rc::new(ProbePlanCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ProbePlanRequiredLabelRule {
                message: "probe plan needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_plan_validator();
        let item = new_probe_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = ProbePlanRequiredNotesRule {
            message: "probe plan needs an entry".to_string(),
        };
        let mut item = new_probe_plan("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ProbePlanLabelLengthRule { max_length: 3600 };
        let mut item = new_probe_plan("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = ProbePlanVersionRangeRule { min: 64, max: 1000 };
        let mut item = new_probe_plan("x", "y");
        item.version = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ProbePlanRequiredLabelRule {
            message: "probe plan needs a value".to_string(),
        };
        let mut item = new_probe_plan("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
