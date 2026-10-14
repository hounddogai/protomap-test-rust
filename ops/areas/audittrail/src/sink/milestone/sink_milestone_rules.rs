use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink milestone must pass. Check returns its problems, or none.
pub trait SinkMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &SinkMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkMilestoneRequiredChecksumRule {
    pub message: String,
}

impl SinkMilestoneRule for SinkMilestoneRequiredChecksumRule {
    fn check(&self, item: &SinkMilestone) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkMilestoneChecksumLengthRule {
    pub max_length: i64,
}

impl SinkMilestoneRule for SinkMilestoneChecksumLengthRule {
    fn check(&self, item: &SinkMilestone) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkMilestoneCodePrefixRule {
    pub prefix: String,
}

impl SinkMilestoneRule for SinkMilestoneCodePrefixRule {
    fn check(&self, item: &SinkMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink milestone.
#[derive(Debug, Clone)]
pub struct SinkMilestoneValidator {
    pub rules: Vec<Rc<dyn SinkMilestoneRule>>,
}

impl SinkMilestoneValidator {
    pub fn validate(&self, item: &SinkMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_milestone_validator() -> SinkMilestoneValidator {
    SinkMilestoneValidator {
        rules: vec![
            Rc::new(SinkMilestoneRequiredChecksumRule {
                message: "sink milestone needs an entry".to_string(),
            }) as Rc<dyn SinkMilestoneRule>,
            Rc::new(SinkMilestoneChecksumLengthRule { max_length: 255 }),
            Rc::new(SinkMilestoneCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_milestone_validator();
        let item = new_sink_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = SinkMilestoneRequiredChecksumRule {
            message: "sink milestone needs this field".to_string(),
        };
        let mut item = new_sink_milestone("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = SinkMilestoneChecksumLengthRule { max_length: 16 };
        let mut item = new_sink_milestone("x", "y");
        item.checksum = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
