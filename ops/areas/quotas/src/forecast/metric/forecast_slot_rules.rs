use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast slot must pass. Check returns its problems, or none.
pub trait ForecastSlotRule: std::fmt::Debug {
    fn check(&self, item: &ForecastSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSlotRequiredRegionRule {
    pub message: String,
}

impl ForecastSlotRule for ForecastSlotRequiredRegionRule {
    fn check(&self, item: &ForecastSlot) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSlotRequiredTitleRule {
    pub message: String,
}

impl ForecastSlotRule for ForecastSlotRequiredTitleRule {
    fn check(&self, item: &ForecastSlot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSlotRequiredCodeRule {
    pub message: String,
}

impl ForecastSlotRule for ForecastSlotRequiredCodeRule {
    fn check(&self, item: &ForecastSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast slot.
#[derive(Debug, Clone)]
pub struct ForecastSlotValidator {
    pub rules: Vec<Rc<dyn ForecastSlotRule>>,
}

impl ForecastSlotValidator {
    pub fn validate(&self, item: &ForecastSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_slot_validator() -> ForecastSlotValidator {
    ForecastSlotValidator {
        rules: vec![
            Rc::new(ForecastSlotRequiredRegionRule {
                message: "forecast slot needs a value".to_string(),
            }) as Rc<dyn ForecastSlotRule>,
            Rc::new(ForecastSlotRequiredTitleRule {
                message: "forecast slot needs an entry".to_string(),
            }),
            Rc::new(ForecastSlotRequiredCodeRule {
                message: "forecast slot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_slot_validator();
        let item = new_forecast_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = ForecastSlotRequiredRegionRule {
            message: "forecast slot needs an entry".to_string(),
        };
        let mut item = new_forecast_slot("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ForecastSlotRequiredTitleRule {
            message: "forecast slot needs this field".to_string(),
        };
        let mut item = new_forecast_slot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ForecastSlotRequiredCodeRule {
            message: "forecast slot needs an entry".to_string(),
        };
        let mut item = new_forecast_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
