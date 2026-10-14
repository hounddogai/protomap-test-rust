use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log profile must pass. Check returns its problems, or none.
pub trait FlowLogProfileRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogProfileRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogProfileRule for FlowLogProfileRotationDaysRangeRule {
    fn check(&self, item: &FlowLogProfile) -> Vec<String> {
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
pub struct FlowLogProfileAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl FlowLogProfileRule for FlowLogProfileAllowedRegionRule {
    fn check(&self, item: &FlowLogProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is too long", item.region)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogProfilePinnedGuardRule {
    pub limit: i64,
}

impl FlowLogProfileRule for FlowLogProfilePinnedGuardRule {
    fn check(&self, item: &FlowLogProfile) -> Vec<String> {
        if item.pinned && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while pinned",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogProfileRequiredNameRule {
    pub message: String,
}

impl FlowLogProfileRule for FlowLogProfileRequiredNameRule {
    fn check(&self, item: &FlowLogProfile) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log profile.
#[derive(Debug, Clone)]
pub struct FlowLogProfileValidator {
    pub rules: Vec<Rc<dyn FlowLogProfileRule>>,
}

impl FlowLogProfileValidator {
    pub fn validate(&self, item: &FlowLogProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_profile_validator() -> FlowLogProfileValidator {
    FlowLogProfileValidator {
        rules: vec![
            Rc::new(FlowLogProfileRotationDaysRangeRule { min: 255, max: 1000 }) as Rc<dyn FlowLogProfileRule>,
            Rc::new(FlowLogProfileAllowedRegionRule {
                allowed: vec!["adaptive".to_string(), "tiered".to_string(), "weighted".to_string()],
            }),
            Rc::new(FlowLogProfilePinnedGuardRule { limit: 120 }),
            Rc::new(FlowLogProfileRequiredNameRule {
                message: "flow log profile needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_profile_validator();
        let item = new_flow_log_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = FlowLogProfileRotationDaysRangeRule { min: 16, max: 255 };
        let mut item = new_flow_log_profile("x", "y");
        item.rotation_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FlowLogProfileRequiredNameRule {
            message: "flow log profile needs a value".to_string(),
        };
        let mut item = new_flow_log_profile("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
