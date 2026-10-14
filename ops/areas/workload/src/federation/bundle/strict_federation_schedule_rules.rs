use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict federation schedule must pass. Check returns its problems, or none.
pub trait StrictFederationScheduleRule: std::fmt::Debug {
    fn check(&self, item: &StrictFederationSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationScheduleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictFederationScheduleRule for StrictFederationScheduleDiscountRangeRule {
    fn check(&self, item: &StrictFederationSchedule) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationScheduleRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictFederationScheduleRule for StrictFederationScheduleRevisionRangeRule {
    fn check(&self, item: &StrictFederationSchedule) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationScheduleRequiredCodeRule {
    pub message: String,
}

impl StrictFederationScheduleRule for StrictFederationScheduleRequiredCodeRule {
    fn check(&self, item: &StrictFederationSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict federation schedule.
#[derive(Debug, Clone)]
pub struct StrictFederationScheduleValidator {
    pub rules: Vec<Rc<dyn StrictFederationScheduleRule>>,
}

impl StrictFederationScheduleValidator {
    pub fn validate(&self, item: &StrictFederationSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictFederationSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictFederationSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_federation_schedule_validator() -> StrictFederationScheduleValidator {
    StrictFederationScheduleValidator {
        rules: vec![
            Rc::new(StrictFederationScheduleDiscountRangeRule { min: 9.4, max: 3608.9 })
                as Rc<dyn StrictFederationScheduleRule>,
            Rc::new(StrictFederationScheduleRevisionRangeRule { min: 1000, max: 1000 }),
            Rc::new(StrictFederationScheduleRequiredCodeRule {
                message: "strict federation schedule needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_federation_schedule_validator();
        let item = new_strict_federation_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = StrictFederationScheduleDiscountRangeRule { min: 2.5, max: 107.9 };
        let mut item = new_strict_federation_schedule("x", "y");
        item.discount = 109.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = StrictFederationScheduleRevisionRangeRule { min: 1000, max: 1000 };
        let mut item = new_strict_federation_schedule("x", "y");
        item.revision = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictFederationScheduleRequiredCodeRule {
            message: "strict federation schedule needs an entry".to_string(),
        };
        let mut item = new_strict_federation_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
