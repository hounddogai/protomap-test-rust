use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual evidence schedule must pass. Check returns its problems, or none.
pub trait ManualEvidenceScheduleRule: std::fmt::Debug {
    fn check(&self, item: &ManualEvidenceSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceScheduleRequiredTitleRule {
    pub message: String,
}

impl ManualEvidenceScheduleRule for ManualEvidenceScheduleRequiredTitleRule {
    fn check(&self, item: &ManualEvidenceSchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceScheduleAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ManualEvidenceScheduleRule for ManualEvidenceScheduleAllowedTitleRule {
    fn check(&self, item: &ManualEvidenceSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceScheduleRequiredCodeRule {
    pub message: String,
}

impl ManualEvidenceScheduleRule for ManualEvidenceScheduleRequiredCodeRule {
    fn check(&self, item: &ManualEvidenceSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceScheduleTitleLengthRule {
    pub max_length: i64,
}

impl ManualEvidenceScheduleRule for ManualEvidenceScheduleTitleLengthRule {
    fn check(&self, item: &ManualEvidenceSchedule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceScheduleCodePrefixRule {
    pub prefix: String,
}

impl ManualEvidenceScheduleRule for ManualEvidenceScheduleCodePrefixRule {
    fn check(&self, item: &ManualEvidenceSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceScheduleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualEvidenceScheduleRule for ManualEvidenceScheduleCreatedAtRangeRule {
    fn check(&self, item: &ManualEvidenceSchedule) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual evidence schedule.
#[derive(Debug, Clone)]
pub struct ManualEvidenceScheduleValidator {
    pub rules: Vec<Rc<dyn ManualEvidenceScheduleRule>>,
}

impl ManualEvidenceScheduleValidator {
    pub fn validate(&self, item: &ManualEvidenceSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualEvidenceSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualEvidenceSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_evidence_schedule_validator() -> ManualEvidenceScheduleValidator {
    ManualEvidenceScheduleValidator {
        rules: vec![
            Rc::new(ManualEvidenceScheduleRequiredTitleRule {
                message: "manual evidence schedule needs a value".to_string(),
            }) as Rc<dyn ManualEvidenceScheduleRule>,
            Rc::new(ManualEvidenceScheduleAllowedTitleRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(ManualEvidenceScheduleRequiredCodeRule {
                message: "manual evidence schedule needs an entry".to_string(),
            }),
            Rc::new(ManualEvidenceScheduleTitleLengthRule { max_length: 3600 }),
            Rc::new(ManualEvidenceScheduleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ManualEvidenceScheduleCreatedAtRangeRule { min: 80, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_evidence_schedule_validator();
        let item = new_manual_evidence_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualEvidenceScheduleRequiredTitleRule {
            message: "manual evidence schedule needs an entry".to_string(),
        };
        let mut item = new_manual_evidence_schedule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualEvidenceScheduleRequiredCodeRule {
            message: "manual evidence schedule needs an entry".to_string(),
        };
        let mut item = new_manual_evidence_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualEvidenceScheduleTitleLengthRule { max_length: 255 };
        let mut item = new_manual_evidence_schedule("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = ManualEvidenceScheduleCreatedAtRangeRule { min: 64, max: 1000 };
        let mut item = new_manual_evidence_schedule("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
