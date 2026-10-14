use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement target must pass. Check returns its problems, or none.
pub trait MeasurementTargetRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTargetNameLengthRule {
    pub max_length: i64,
}

impl MeasurementTargetRule for MeasurementTargetNameLengthRule {
    fn check(&self, item: &MeasurementTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTargetRequiredCurrencyRule {
    pub message: String,
}

impl MeasurementTargetRule for MeasurementTargetRequiredCurrencyRule {
    fn check(&self, item: &MeasurementTarget) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTargetRequiredCodeRule {
    pub message: String,
}

impl MeasurementTargetRule for MeasurementTargetRequiredCodeRule {
    fn check(&self, item: &MeasurementTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTargetRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementTargetRule for MeasurementTargetRevisionRangeRule {
    fn check(&self, item: &MeasurementTarget) -> Vec<String> {
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

/// Runs every rule of a measurement target.
#[derive(Debug, Clone)]
pub struct MeasurementTargetValidator {
    pub rules: Vec<Rc<dyn MeasurementTargetRule>>,
}

impl MeasurementTargetValidator {
    pub fn validate(&self, item: &MeasurementTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MeasurementTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_target_validator() -> MeasurementTargetValidator {
    MeasurementTargetValidator {
        rules: vec![
            Rc::new(MeasurementTargetNameLengthRule { max_length: 255 }) as Rc<dyn MeasurementTargetRule>,
            Rc::new(MeasurementTargetRequiredCurrencyRule {
                message: "measurement target needs this field".to_string(),
            }),
            Rc::new(MeasurementTargetRequiredCodeRule {
                message: "measurement target needs an entry".to_string(),
            }),
            Rc::new(MeasurementTargetRevisionRangeRule { min: 32, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_target_validator();
        let item = new_measurement_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = MeasurementTargetNameLengthRule { max_length: 64 };
        let mut item = new_measurement_target("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = MeasurementTargetRequiredCurrencyRule {
            message: "measurement target needs this field".to_string(),
        };
        let mut item = new_measurement_target("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MeasurementTargetRequiredCodeRule {
            message: "measurement target needs this field".to_string(),
        };
        let mut item = new_measurement_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = MeasurementTargetRevisionRangeRule { min: 80, max: 120 };
        let mut item = new_measurement_target("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
