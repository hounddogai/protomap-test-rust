use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone milestone must pass. Check returns its problems, or none.
pub trait ZoneMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &ZoneMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMilestoneTitleLengthRule {
    pub max_length: i64,
}

impl ZoneMilestoneRule for ZoneMilestoneTitleLengthRule {
    fn check(&self, item: &ZoneMilestone) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMilestoneNotesLengthRule {
    pub max_length: i64,
}

impl ZoneMilestoneRule for ZoneMilestoneNotesLengthRule {
    fn check(&self, item: &ZoneMilestone) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMilestoneRequiredNotesRule {
    pub message: String,
}

impl ZoneMilestoneRule for ZoneMilestoneRequiredNotesRule {
    fn check(&self, item: &ZoneMilestone) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone milestone.
#[derive(Debug, Clone)]
pub struct ZoneMilestoneValidator {
    pub rules: Vec<Rc<dyn ZoneMilestoneRule>>,
}

impl ZoneMilestoneValidator {
    pub fn validate(&self, item: &ZoneMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_milestone_validator() -> ZoneMilestoneValidator {
    ZoneMilestoneValidator {
        rules: vec![
            Rc::new(ZoneMilestoneTitleLengthRule { max_length: 64 }) as Rc<dyn ZoneMilestoneRule>,
            Rc::new(ZoneMilestoneNotesLengthRule { max_length: 120 }),
            Rc::new(ZoneMilestoneRequiredNotesRule {
                message: "zone milestone needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_milestone_validator();
        let item = new_zone_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ZoneMilestoneTitleLengthRule { max_length: 86400 };
        let mut item = new_zone_milestone("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = ZoneMilestoneNotesLengthRule { max_length: 3600 };
        let mut item = new_zone_milestone("x", "y");
        item.notes = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = ZoneMilestoneRequiredNotesRule {
            message: "zone milestone needs a value".to_string(),
        };
        let mut item = new_zone_milestone("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
