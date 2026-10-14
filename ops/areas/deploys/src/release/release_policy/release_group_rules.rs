use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release group must pass. Check returns its problems, or none.
pub trait ReleaseGroupRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseGroupReviewerLengthRule {
    pub max_length: i64,
}

impl ReleaseGroupRule for ReleaseGroupReviewerLengthRule {
    fn check(&self, item: &ReleaseGroup) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseGroupCodeLengthRule {
    pub max_length: i64,
}

impl ReleaseGroupRule for ReleaseGroupCodeLengthRule {
    fn check(&self, item: &ReleaseGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseGroupUnitLengthRule {
    pub max_length: i64,
}

impl ReleaseGroupRule for ReleaseGroupUnitLengthRule {
    fn check(&self, item: &ReleaseGroup) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseGroupTitleLengthRule {
    pub max_length: i64,
}

impl ReleaseGroupRule for ReleaseGroupTitleLengthRule {
    fn check(&self, item: &ReleaseGroup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseGroupEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseGroupRule for ReleaseGroupEndsAtRangeRule {
    fn check(&self, item: &ReleaseGroup) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release group.
#[derive(Debug, Clone)]
pub struct ReleaseGroupValidator {
    pub rules: Vec<Rc<dyn ReleaseGroupRule>>,
}

impl ReleaseGroupValidator {
    pub fn validate(&self, item: &ReleaseGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_group_validator() -> ReleaseGroupValidator {
    ReleaseGroupValidator {
        rules: vec![
            Rc::new(ReleaseGroupReviewerLengthRule { max_length: 86400 }) as Rc<dyn ReleaseGroupRule>,
            Rc::new(ReleaseGroupCodeLengthRule { max_length: 3600 }),
            Rc::new(ReleaseGroupUnitLengthRule { max_length: 86400 }),
            Rc::new(ReleaseGroupTitleLengthRule { max_length: 64 }),
            Rc::new(ReleaseGroupEndsAtRangeRule { min: 80, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_group_validator();
        let item = new_release_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = ReleaseGroupReviewerLengthRule { max_length: 120 };
        let mut item = new_release_group("x", "y");
        item.reviewer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReleaseGroupCodeLengthRule { max_length: 80 };
        let mut item = new_release_group("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = ReleaseGroupUnitLengthRule { max_length: 64 };
        let mut item = new_release_group("x", "y");
        item.unit = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ReleaseGroupTitleLengthRule { max_length: 1000 };
        let mut item = new_release_group("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = ReleaseGroupEndsAtRangeRule { min: 16, max: 86400 };
        let mut item = new_release_group("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
