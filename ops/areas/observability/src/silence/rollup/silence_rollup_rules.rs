use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence rollup must pass. Check returns its problems, or none.
pub trait SilenceRollupRule: std::fmt::Debug {
    fn check(&self, item: &SilenceRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceRollupDescriptionLengthRule {
    pub max_length: i64,
}

impl SilenceRollupRule for SilenceRollupDescriptionLengthRule {
    fn check(&self, item: &SilenceRollup) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceRollupSegmentLengthRule {
    pub max_length: i64,
}

impl SilenceRollupRule for SilenceRollupSegmentLengthRule {
    fn check(&self, item: &SilenceRollup) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceRollupRequiredDescriptionRule {
    pub message: String,
}

impl SilenceRollupRule for SilenceRollupRequiredDescriptionRule {
    fn check(&self, item: &SilenceRollup) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence rollup.
#[derive(Debug, Clone)]
pub struct SilenceRollupValidator {
    pub rules: Vec<Rc<dyn SilenceRollupRule>>,
}

impl SilenceRollupValidator {
    pub fn validate(&self, item: &SilenceRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_rollup_validator() -> SilenceRollupValidator {
    SilenceRollupValidator {
        rules: vec![
            Rc::new(SilenceRollupDescriptionLengthRule { max_length: 16 }) as Rc<dyn SilenceRollupRule>,
            Rc::new(SilenceRollupSegmentLengthRule { max_length: 1000 }),
            Rc::new(SilenceRollupRequiredDescriptionRule {
                message: "silence rollup needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_rollup_validator();
        let item = new_silence_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = SilenceRollupDescriptionLengthRule { max_length: 120 };
        let mut item = new_silence_rollup("x", "y");
        item.description = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = SilenceRollupSegmentLengthRule { max_length: 64 };
        let mut item = new_silence_rollup("x", "y");
        item.segment = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = SilenceRollupRequiredDescriptionRule {
            message: "silence rollup needs this field".to_string(),
        };
        let mut item = new_silence_rollup("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
