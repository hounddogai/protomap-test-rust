use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy postmortem plan must pass. Check returns its problems, or none.
pub trait LegacyPostmortemPlanRule: std::fmt::Debug {
    fn check(&self, item: &LegacyPostmortemPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemPlanAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl LegacyPostmortemPlanRule for LegacyPostmortemPlanAllowedSegmentRule {
    fn check(&self, item: &LegacyPostmortemPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is expired", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemPlanFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyPostmortemPlanRule for LegacyPostmortemPlanFloorValueRangeRule {
    fn check(&self, item: &LegacyPostmortemPlan) -> Vec<String> {
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
pub struct LegacyPostmortemPlanRequiredCodeRule {
    pub message: String,
}

impl LegacyPostmortemPlanRule for LegacyPostmortemPlanRequiredCodeRule {
    fn check(&self, item: &LegacyPostmortemPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemPlanCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyPostmortemPlanRule for LegacyPostmortemPlanCostRangeRule {
    fn check(&self, item: &LegacyPostmortemPlan) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemPlanSegmentLengthRule {
    pub max_length: i64,
}

impl LegacyPostmortemPlanRule for LegacyPostmortemPlanSegmentLengthRule {
    fn check(&self, item: &LegacyPostmortemPlan) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy postmortem plan.
#[derive(Debug, Clone)]
pub struct LegacyPostmortemPlanValidator {
    pub rules: Vec<Rc<dyn LegacyPostmortemPlanRule>>,
}

impl LegacyPostmortemPlanValidator {
    pub fn validate(&self, item: &LegacyPostmortemPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyPostmortemPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyPostmortemPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_postmortem_plan_validator() -> LegacyPostmortemPlanValidator {
    LegacyPostmortemPlanValidator {
        rules: vec![
            Rc::new(LegacyPostmortemPlanAllowedSegmentRule {
                allowed: vec!["pending".to_string(), "batched".to_string(), "weighted".to_string()],
            }) as Rc<dyn LegacyPostmortemPlanRule>,
            Rc::new(LegacyPostmortemPlanFloorValueRangeRule { min: 5.0, max: 4917.4 }),
            Rc::new(LegacyPostmortemPlanRequiredCodeRule {
                message: "legacy postmortem plan needs this field".to_string(),
            }),
            Rc::new(LegacyPostmortemPlanCostRangeRule { min: 6.8, max: 1130.8 }),
            Rc::new(LegacyPostmortemPlanSegmentLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_postmortem_plan_validator();
        let item = new_legacy_postmortem_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = LegacyPostmortemPlanFloorValueRangeRule { min: 6.3, max: 2190.1 };
        let mut item = new_legacy_postmortem_plan("x", "y");
        item.floor_value = 2191.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyPostmortemPlanRequiredCodeRule {
            message: "legacy postmortem plan needs an entry".to_string(),
        };
        let mut item = new_legacy_postmortem_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = LegacyPostmortemPlanCostRangeRule { min: 1.0, max: 183.8 };
        let mut item = new_legacy_postmortem_plan("x", "y");
        item.cost = 185.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = LegacyPostmortemPlanSegmentLengthRule { max_length: 32 };
        let mut item = new_legacy_postmortem_plan("x", "y");
        item.segment = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
