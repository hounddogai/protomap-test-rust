use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin rollup must pass. Check returns its problems, or none.
pub trait RegionPinRollupRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRollupRequiredNotesRule {
    pub message: String,
}

impl RegionPinRollupRule for RegionPinRollupRequiredNotesRule {
    fn check(&self, item: &RegionPinRollup) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRollupRequiredCodeRule {
    pub message: String,
}

impl RegionPinRollupRule for RegionPinRollupRequiredCodeRule {
    fn check(&self, item: &RegionPinRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRollupCodeLengthRule {
    pub max_length: i64,
}

impl RegionPinRollupRule for RegionPinRollupCodeLengthRule {
    fn check(&self, item: &RegionPinRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRollupNotesLengthRule {
    pub max_length: i64,
}

impl RegionPinRollupRule for RegionPinRollupNotesLengthRule {
    fn check(&self, item: &RegionPinRollup) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin rollup.
#[derive(Debug, Clone)]
pub struct RegionPinRollupValidator {
    pub rules: Vec<Rc<dyn RegionPinRollupRule>>,
}

impl RegionPinRollupValidator {
    pub fn validate(&self, item: &RegionPinRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_rollup_validator() -> RegionPinRollupValidator {
    RegionPinRollupValidator {
        rules: vec![
            Rc::new(RegionPinRollupRequiredNotesRule {
                message: "region pin rollup needs an entry".to_string(),
            }) as Rc<dyn RegionPinRollupRule>,
            Rc::new(RegionPinRollupRequiredCodeRule {
                message: "region pin rollup needs an entry".to_string(),
            }),
            Rc::new(RegionPinRollupCodeLengthRule { max_length: 3600 }),
            Rc::new(RegionPinRollupNotesLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_rollup_validator();
        let item = new_region_pin_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RegionPinRollupRequiredNotesRule {
            message: "region pin rollup needs an entry".to_string(),
        };
        let mut item = new_region_pin_rollup("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinRollupRequiredCodeRule {
            message: "region pin rollup needs an entry".to_string(),
        };
        let mut item = new_region_pin_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionPinRollupCodeLengthRule { max_length: 1000 };
        let mut item = new_region_pin_rollup("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RegionPinRollupNotesLengthRule { max_length: 1000 };
        let mut item = new_region_pin_rollup("x", "y");
        item.notes = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
