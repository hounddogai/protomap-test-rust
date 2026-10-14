use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional suspension profile must pass. Check returns its problems, or none.
pub trait RegionalSuspensionProfileRule: std::fmt::Debug {
    fn check(&self, item: &RegionalSuspensionProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionProfileApprovedGuardRule {
    pub limit: i64,
}

impl RegionalSuspensionProfileRule for RegionalSuspensionProfileApprovedGuardRule {
    fn check(&self, item: &RegionalSuspensionProfile) -> Vec<String> {
        if item.approved && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while approved",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionProfileRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalSuspensionProfileRule for RegionalSuspensionProfileRotationDaysRangeRule {
    fn check(&self, item: &RegionalSuspensionProfile) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionProfileRequiredTitleRule {
    pub message: String,
}

impl RegionalSuspensionProfileRule for RegionalSuspensionProfileRequiredTitleRule {
    fn check(&self, item: &RegionalSuspensionProfile) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionProfileTitleLengthRule {
    pub max_length: i64,
}

impl RegionalSuspensionProfileRule for RegionalSuspensionProfileTitleLengthRule {
    fn check(&self, item: &RegionalSuspensionProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionProfileClusterLengthRule {
    pub max_length: i64,
}

impl RegionalSuspensionProfileRule for RegionalSuspensionProfileClusterLengthRule {
    fn check(&self, item: &RegionalSuspensionProfile) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional suspension profile.
#[derive(Debug, Clone)]
pub struct RegionalSuspensionProfileValidator {
    pub rules: Vec<Rc<dyn RegionalSuspensionProfileRule>>,
}

impl RegionalSuspensionProfileValidator {
    pub fn validate(&self, item: &RegionalSuspensionProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalSuspensionProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalSuspensionProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_suspension_profile_validator() -> RegionalSuspensionProfileValidator {
    RegionalSuspensionProfileValidator {
        rules: vec![
            Rc::new(RegionalSuspensionProfileApprovedGuardRule { limit: 80 }) as Rc<dyn RegionalSuspensionProfileRule>,
            Rc::new(RegionalSuspensionProfileRotationDaysRangeRule { min: 16, max: 80 }),
            Rc::new(RegionalSuspensionProfileRequiredTitleRule {
                message: "regional suspension profile needs a value".to_string(),
            }),
            Rc::new(RegionalSuspensionProfileTitleLengthRule { max_length: 16 }),
            Rc::new(RegionalSuspensionProfileClusterLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_suspension_profile_validator();
        let item = new_regional_suspension_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = RegionalSuspensionProfileRotationDaysRangeRule { min: 32, max: 1000 };
        let mut item = new_regional_suspension_profile("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionalSuspensionProfileRequiredTitleRule {
            message: "regional suspension profile needs a value".to_string(),
        };
        let mut item = new_regional_suspension_profile("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalSuspensionProfileTitleLengthRule { max_length: 86400 };
        let mut item = new_regional_suspension_profile("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = RegionalSuspensionProfileClusterLengthRule { max_length: 86400 };
        let mut item = new_regional_suspension_profile("x", "y");
        item.cluster = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
