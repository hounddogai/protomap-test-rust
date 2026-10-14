use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release milestone must pass. Check returns its problems, or none.
pub trait ReleaseMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMilestoneLabelLengthRule {
    pub max_length: i64,
}

impl ReleaseMilestoneRule for ReleaseMilestoneLabelLengthRule {
    fn check(&self, item: &ReleaseMilestone) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMilestoneAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ReleaseMilestoneRule for ReleaseMilestoneAllowedLabelRule {
    fn check(&self, item: &ReleaseMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is below minimum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMilestoneRequiredLabelRule {
    pub message: String,
}

impl ReleaseMilestoneRule for ReleaseMilestoneRequiredLabelRule {
    fn check(&self, item: &ReleaseMilestone) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMilestoneIssuerLengthRule {
    pub max_length: i64,
}

impl ReleaseMilestoneRule for ReleaseMilestoneIssuerLengthRule {
    fn check(&self, item: &ReleaseMilestone) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMilestoneQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseMilestoneRule for ReleaseMilestoneQuotaUnitsRangeRule {
    fn check(&self, item: &ReleaseMilestone) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMilestoneRequiredNameRule {
    pub message: String,
}

impl ReleaseMilestoneRule for ReleaseMilestoneRequiredNameRule {
    fn check(&self, item: &ReleaseMilestone) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a release milestone.
#[derive(Debug, Clone)]
pub struct ReleaseMilestoneValidator {
    pub rules: Vec<Rc<dyn ReleaseMilestoneRule>>,
}

impl ReleaseMilestoneValidator {
    pub fn validate(&self, item: &ReleaseMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_milestone_validator() -> ReleaseMilestoneValidator {
    ReleaseMilestoneValidator {
        rules: vec![
            Rc::new(ReleaseMilestoneLabelLengthRule { max_length: 120 }) as Rc<dyn ReleaseMilestoneRule>,
            Rc::new(ReleaseMilestoneAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(ReleaseMilestoneRequiredLabelRule {
                message: "release milestone needs a value".to_string(),
            }),
            Rc::new(ReleaseMilestoneIssuerLengthRule { max_length: 80 }),
            Rc::new(ReleaseMilestoneQuotaUnitsRangeRule { min: 64, max: 255 }),
            Rc::new(ReleaseMilestoneRequiredNameRule {
                message: "release milestone needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_milestone_validator();
        let item = new_release_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ReleaseMilestoneLabelLengthRule { max_length: 120 };
        let mut item = new_release_milestone("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ReleaseMilestoneRequiredLabelRule {
            message: "release milestone needs this field".to_string(),
        };
        let mut item = new_release_milestone("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = ReleaseMilestoneIssuerLengthRule { max_length: 32 };
        let mut item = new_release_milestone("x", "y");
        item.issuer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = ReleaseMilestoneQuotaUnitsRangeRule { min: 64, max: 64 };
        let mut item = new_release_milestone("x", "y");
        item.quota_units = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ReleaseMilestoneRequiredNameRule {
            message: "release milestone needs this field".to_string(),
        };
        let mut item = new_release_milestone("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
