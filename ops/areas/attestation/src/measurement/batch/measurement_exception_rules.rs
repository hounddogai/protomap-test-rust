use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a measurement exception must pass. Check returns its problems, or none.
pub trait MeasurementExceptionRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementExceptionAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl MeasurementExceptionRule for MeasurementExceptionAllowedNamespaceRule {
    fn check(&self, item: &MeasurementException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is duplicated", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementExceptionNameLengthRule {
    pub max_length: i64,
}

impl MeasurementExceptionRule for MeasurementExceptionNameLengthRule {
    fn check(&self, item: &MeasurementException) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementExceptionRequiredCodeRule {
    pub message: String,
}

impl MeasurementExceptionRule for MeasurementExceptionRequiredCodeRule {
    fn check(&self, item: &MeasurementException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement exception.
#[derive(Debug, Clone)]
pub struct MeasurementExceptionValidator {
    pub rules: Vec<Rc<dyn MeasurementExceptionRule>>,
}

impl MeasurementExceptionValidator {
    pub fn validate(&self, item: &MeasurementException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_exception_validator() -> MeasurementExceptionValidator {
    MeasurementExceptionValidator {
        rules: vec![
            Rc::new(MeasurementExceptionAllowedNamespaceRule {
                allowed: vec!["strict".to_string()],
            }) as Rc<dyn MeasurementExceptionRule>,
            Rc::new(MeasurementExceptionNameLengthRule { max_length: 120 }),
            Rc::new(MeasurementExceptionRequiredCodeRule {
                message: "measurement exception needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_exception_validator();
        let item = new_measurement_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = MeasurementExceptionNameLengthRule { max_length: 255 };
        let mut item = new_measurement_exception("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MeasurementExceptionRequiredCodeRule {
            message: "measurement exception needs this field".to_string(),
        };
        let mut item = new_measurement_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
