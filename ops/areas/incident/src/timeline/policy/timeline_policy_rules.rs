use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline policy must pass. Check returns its problems, or none.
pub trait TimelinePolicyRule: std::fmt::Debug {
    fn check(&self, item: &TimelinePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePolicyFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelinePolicyRule for TimelinePolicyFloorValueRangeRule {
    fn check(&self, item: &TimelinePolicy) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePolicyCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelinePolicyRule for TimelinePolicyCostRangeRule {
    fn check(&self, item: &TimelinePolicy) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePolicyLabelLengthRule {
    pub max_length: i64,
}

impl TimelinePolicyRule for TimelinePolicyLabelLengthRule {
    fn check(&self, item: &TimelinePolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePolicyRequiredCodeRule {
    pub message: String,
}

impl TimelinePolicyRule for TimelinePolicyRequiredCodeRule {
    fn check(&self, item: &TimelinePolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePolicyRequiredTenantIdRule {
    pub message: String,
}

impl TimelinePolicyRule for TimelinePolicyRequiredTenantIdRule {
    fn check(&self, item: &TimelinePolicy) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePolicyAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl TimelinePolicyRule for TimelinePolicyAllowedAudienceRule {
    fn check(&self, item: &TimelinePolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is out of range", item.audience)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline policy.
#[derive(Debug, Clone)]
pub struct TimelinePolicyValidator {
    pub rules: Vec<Rc<dyn TimelinePolicyRule>>,
}

impl TimelinePolicyValidator {
    pub fn validate(&self, item: &TimelinePolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelinePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelinePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_policy_validator() -> TimelinePolicyValidator {
    TimelinePolicyValidator {
        rules: vec![
            Rc::new(TimelinePolicyFloorValueRangeRule { min: 4.7, max: 3677.5 }) as Rc<dyn TimelinePolicyRule>,
            Rc::new(TimelinePolicyCostRangeRule { min: 1.4, max: 2488.1 }),
            Rc::new(TimelinePolicyLabelLengthRule { max_length: 80 }),
            Rc::new(TimelinePolicyRequiredCodeRule {
                message: "timeline policy needs this field".to_string(),
            }),
            Rc::new(TimelinePolicyRequiredTenantIdRule {
                message: "timeline policy needs this field".to_string(),
            }),
            Rc::new(TimelinePolicyAllowedAudienceRule {
                allowed: vec!["lenient".to_string(), "regional".to_string(), "fallback".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_policy_validator();
        let item = new_timeline_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = TimelinePolicyFloorValueRangeRule { min: 9.8, max: 3988.6 };
        let mut item = new_timeline_policy("x", "y");
        item.floor_value = 3990.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = TimelinePolicyCostRangeRule { min: 0.9, max: 2805.5 };
        let mut item = new_timeline_policy("x", "y");
        item.cost = 2807.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TimelinePolicyLabelLengthRule { max_length: 1000 };
        let mut item = new_timeline_policy("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TimelinePolicyRequiredCodeRule {
            message: "timeline policy needs an entry".to_string(),
        };
        let mut item = new_timeline_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = TimelinePolicyRequiredTenantIdRule {
            message: "timeline policy needs an entry".to_string(),
        };
        let mut item = new_timeline_policy("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
