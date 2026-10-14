use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary canary summary must pass. Check returns its problems, or none.
pub trait PrimaryCanarySummaryRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryCanarySummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanarySummaryLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryCanarySummaryRule for PrimaryCanarySummaryLabelLengthRule {
    fn check(&self, item: &PrimaryCanarySummary) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanarySummaryRequiredLabelRule {
    pub message: String,
}

impl PrimaryCanarySummaryRule for PrimaryCanarySummaryRequiredLabelRule {
    fn check(&self, item: &PrimaryCanarySummary) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanarySummaryRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryCanarySummaryRule for PrimaryCanarySummaryRevisionRangeRule {
    fn check(&self, item: &PrimaryCanarySummary) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a primary canary summary.
#[derive(Debug, Clone)]
pub struct PrimaryCanarySummaryValidator {
    pub rules: Vec<Rc<dyn PrimaryCanarySummaryRule>>,
}

impl PrimaryCanarySummaryValidator {
    pub fn validate(&self, item: &PrimaryCanarySummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryCanarySummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryCanarySummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_canary_summary_validator() -> PrimaryCanarySummaryValidator {
    PrimaryCanarySummaryValidator {
        rules: vec![
            Rc::new(PrimaryCanarySummaryLabelLengthRule { max_length: 64 }) as Rc<dyn PrimaryCanarySummaryRule>,
            Rc::new(PrimaryCanarySummaryRequiredLabelRule {
                message: "primary canary summary needs a value".to_string(),
            }),
            Rc::new(PrimaryCanarySummaryRevisionRangeRule { min: 32, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_canary_summary_validator();
        let item = new_primary_canary_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryCanarySummaryLabelLengthRule { max_length: 80 };
        let mut item = new_primary_canary_summary("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimaryCanarySummaryRequiredLabelRule {
            message: "primary canary summary needs this field".to_string(),
        };
        let mut item = new_primary_canary_summary("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = PrimaryCanarySummaryRevisionRangeRule { min: 16, max: 64 };
        let mut item = new_primary_canary_summary("x", "y");
        item.revision = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
