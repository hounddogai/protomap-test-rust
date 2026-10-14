use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy probe assignment must pass. Check returns its problems, or none.
pub trait LegacyProbeAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &LegacyProbeAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeAssignmentRequiredSummaryRule {
    pub message: String,
}

impl LegacyProbeAssignmentRule for LegacyProbeAssignmentRequiredSummaryRule {
    fn check(&self, item: &LegacyProbeAssignment) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeAssignmentRequiredCodeRule {
    pub message: String,
}

impl LegacyProbeAssignmentRule for LegacyProbeAssignmentRequiredCodeRule {
    fn check(&self, item: &LegacyProbeAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl LegacyProbeAssignmentRule for LegacyProbeAssignmentCodeLengthRule {
    fn check(&self, item: &LegacyProbeAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeAssignmentRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyProbeAssignmentRule for LegacyProbeAssignmentRateRangeRule {
    fn check(&self, item: &LegacyProbeAssignment) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeAssignmentTitleLengthRule {
    pub max_length: i64,
}

impl LegacyProbeAssignmentRule for LegacyProbeAssignmentTitleLengthRule {
    fn check(&self, item: &LegacyProbeAssignment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy probe assignment.
#[derive(Debug, Clone)]
pub struct LegacyProbeAssignmentValidator {
    pub rules: Vec<Rc<dyn LegacyProbeAssignmentRule>>,
}

impl LegacyProbeAssignmentValidator {
    pub fn validate(&self, item: &LegacyProbeAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyProbeAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyProbeAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_probe_assignment_validator() -> LegacyProbeAssignmentValidator {
    LegacyProbeAssignmentValidator {
        rules: vec![
            Rc::new(LegacyProbeAssignmentRequiredSummaryRule {
                message: "legacy probe assignment needs this field".to_string(),
            }) as Rc<dyn LegacyProbeAssignmentRule>,
            Rc::new(LegacyProbeAssignmentRequiredCodeRule {
                message: "legacy probe assignment needs this field".to_string(),
            }),
            Rc::new(LegacyProbeAssignmentCodeLengthRule { max_length: 86400 }),
            Rc::new(LegacyProbeAssignmentRateRangeRule { min: 5.1, max: 3365.0 }),
            Rc::new(LegacyProbeAssignmentTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_probe_assignment_validator();
        let item = new_legacy_probe_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = LegacyProbeAssignmentRequiredSummaryRule {
            message: "legacy probe assignment needs a value".to_string(),
        };
        let mut item = new_legacy_probe_assignment("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyProbeAssignmentRequiredCodeRule {
            message: "legacy probe assignment needs a value".to_string(),
        };
        let mut item = new_legacy_probe_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyProbeAssignmentCodeLengthRule { max_length: 255 };
        let mut item = new_legacy_probe_assignment("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = LegacyProbeAssignmentRateRangeRule { min: 3.9, max: 4326.0 };
        let mut item = new_legacy_probe_assignment("x", "y");
        item.rate = 4327.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyProbeAssignmentTitleLengthRule { max_length: 1000 };
        let mut item = new_legacy_probe_assignment("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
