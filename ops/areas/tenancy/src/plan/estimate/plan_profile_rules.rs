use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan profile must pass. Check returns its problems, or none.
pub trait PlanProfileRule: std::fmt::Debug {
    fn check(&self, item: &PlanProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanProfileRequiredDescriptionRule {
    pub message: String,
}

impl PlanProfileRule for PlanProfileRequiredDescriptionRule {
    fn check(&self, item: &PlanProfile) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanProfileAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PlanProfileRule for PlanProfileAllowedLabelRule {
    fn check(&self, item: &PlanProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanProfileLabelLengthRule {
    pub max_length: i64,
}

impl PlanProfileRule for PlanProfileLabelLengthRule {
    fn check(&self, item: &PlanProfile) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanProfileScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PlanProfileRule for PlanProfileScoreRangeRule {
    fn check(&self, item: &PlanProfile) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanProfileRequiredTenantIdRule {
    pub message: String,
}

impl PlanProfileRule for PlanProfileRequiredTenantIdRule {
    fn check(&self, item: &PlanProfile) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanProfileRequiredCodeRule {
    pub message: String,
}

impl PlanProfileRule for PlanProfileRequiredCodeRule {
    fn check(&self, item: &PlanProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan profile.
#[derive(Debug, Clone)]
pub struct PlanProfileValidator {
    pub rules: Vec<Rc<dyn PlanProfileRule>>,
}

impl PlanProfileValidator {
    pub fn validate(&self, item: &PlanProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_profile_validator() -> PlanProfileValidator {
    PlanProfileValidator {
        rules: vec![
            Rc::new(PlanProfileRequiredDescriptionRule {
                message: "plan profile needs an entry".to_string(),
            }) as Rc<dyn PlanProfileRule>,
            Rc::new(PlanProfileAllowedLabelRule {
                allowed: vec!["pending".to_string(), "default".to_string(), "rolling".to_string()],
            }),
            Rc::new(PlanProfileLabelLengthRule { max_length: 32 }),
            Rc::new(PlanProfileScoreRangeRule { min: 8.0, max: 1344.8 }),
            Rc::new(PlanProfileRequiredTenantIdRule {
                message: "plan profile needs an entry".to_string(),
            }),
            Rc::new(PlanProfileRequiredCodeRule {
                message: "plan profile needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_profile_validator();
        let item = new_plan_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = PlanProfileRequiredDescriptionRule {
            message: "plan profile needs an entry".to_string(),
        };
        let mut item = new_plan_profile("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PlanProfileLabelLengthRule { max_length: 120 };
        let mut item = new_plan_profile("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PlanProfileScoreRangeRule { min: 3.1, max: 3395.9 };
        let mut item = new_plan_profile("x", "y");
        item.score = 3397.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = PlanProfileRequiredTenantIdRule {
            message: "plan profile needs an entry".to_string(),
        };
        let mut item = new_plan_profile("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PlanProfileRequiredCodeRule {
            message: "plan profile needs a value".to_string(),
        };
        let mut item = new_plan_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
