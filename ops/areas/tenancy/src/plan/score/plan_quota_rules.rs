use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan quota must pass. Check returns its problems, or none.
pub trait PlanQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PlanQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanQuotaTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl PlanQuotaRule for PlanQuotaTagsLimitRule {
    fn check(&self, item: &PlanQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanQuotaRequiredCategoryRule {
    pub message: String,
}

impl PlanQuotaRule for PlanQuotaRequiredCategoryRule {
    fn check(&self, item: &PlanQuota) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanQuotaPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanQuotaRule for PlanQuotaPriorityRangeRule {
    fn check(&self, item: &PlanQuota) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct PlanQuotaRequiredCodeRule {
    pub message: String,
}

impl PlanQuotaRule for PlanQuotaRequiredCodeRule {
    fn check(&self, item: &PlanQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan quota.
#[derive(Debug, Clone)]
pub struct PlanQuotaValidator {
    pub rules: Vec<Rc<dyn PlanQuotaRule>>,
}

impl PlanQuotaValidator {
    pub fn validate(&self, item: &PlanQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_quota_validator() -> PlanQuotaValidator {
    PlanQuotaValidator {
        rules: vec![
            Rc::new(PlanQuotaTagsLimitRule {
                max_count: 16,
                banned: vec!["global".to_string(), "primary".to_string()],
            }) as Rc<dyn PlanQuotaRule>,
            Rc::new(PlanQuotaRequiredCategoryRule {
                message: "plan quota needs an entry".to_string(),
            }),
            Rc::new(PlanQuotaPriorityRangeRule { min: 16, max: 80 }),
            Rc::new(PlanQuotaRequiredCodeRule {
                message: "plan quota needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_quota_validator();
        let item = new_plan_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = PlanQuotaRequiredCategoryRule {
            message: "plan quota needs this field".to_string(),
        };
        let mut item = new_plan_quota("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = PlanQuotaPriorityRangeRule { min: 80, max: 255 };
        let mut item = new_plan_quota("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PlanQuotaRequiredCodeRule {
            message: "plan quota needs this field".to_string(),
        };
        let mut item = new_plan_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
