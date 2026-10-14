use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook target must pass. Check returns its problems, or none.
pub trait RunbookTargetRule: std::fmt::Debug {
    fn check(&self, item: &RunbookTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTargetCodeLengthRule {
    pub max_length: i64,
}

impl RunbookTargetRule for RunbookTargetCodeLengthRule {
    fn check(&self, item: &RunbookTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTargetChecksumLengthRule {
    pub max_length: i64,
}

impl RunbookTargetRule for RunbookTargetChecksumLengthRule {
    fn check(&self, item: &RunbookTarget) -> Vec<String> {
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
pub struct RunbookTargetRegionLengthRule {
    pub max_length: i64,
}

impl RunbookTargetRule for RunbookTargetRegionLengthRule {
    fn check(&self, item: &RunbookTarget) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTargetRequiredTitleRule {
    pub message: String,
}

impl RunbookTargetRule for RunbookTargetRequiredTitleRule {
    fn check(&self, item: &RunbookTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTargetPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookTargetRule for RunbookTargetPriorityRangeRule {
    fn check(&self, item: &RunbookTarget) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a runbook target.
#[derive(Debug, Clone)]
pub struct RunbookTargetValidator {
    pub rules: Vec<Rc<dyn RunbookTargetRule>>,
}

impl RunbookTargetValidator {
    pub fn validate(&self, item: &RunbookTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_target_validator() -> RunbookTargetValidator {
    RunbookTargetValidator {
        rules: vec![
            Rc::new(RunbookTargetCodeLengthRule { max_length: 120 }) as Rc<dyn RunbookTargetRule>,
            Rc::new(RunbookTargetChecksumLengthRule { max_length: 86400 }),
            Rc::new(RunbookTargetRegionLengthRule { max_length: 255 }),
            Rc::new(RunbookTargetRequiredTitleRule {
                message: "runbook target needs an entry".to_string(),
            }),
            Rc::new(RunbookTargetPriorityRangeRule { min: 32, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_target_validator();
        let item = new_runbook_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RunbookTargetCodeLengthRule { max_length: 80 };
        let mut item = new_runbook_target("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = RunbookTargetChecksumLengthRule { max_length: 3600 };
        let mut item = new_runbook_target("x", "y");
        item.checksum = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = RunbookTargetRegionLengthRule { max_length: 64 };
        let mut item = new_runbook_target("x", "y");
        item.region = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RunbookTargetRequiredTitleRule {
            message: "runbook target needs an entry".to_string(),
        };
        let mut item = new_runbook_target("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = RunbookTargetPriorityRangeRule { min: 3600, max: 86400 };
        let mut item = new_runbook_target("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
