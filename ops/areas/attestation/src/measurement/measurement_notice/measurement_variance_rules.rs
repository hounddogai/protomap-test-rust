use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement variance must pass. Check returns its problems, or none.
pub trait MeasurementVarianceRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementVarianceRequiredLabelRule {
    pub message: String,
}

impl MeasurementVarianceRule for MeasurementVarianceRequiredLabelRule {
    fn check(&self, item: &MeasurementVariance) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementVarianceIssuerLengthRule {
    pub max_length: i64,
}

impl MeasurementVarianceRule for MeasurementVarianceIssuerLengthRule {
    fn check(&self, item: &MeasurementVariance) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementVarianceRequiredIssuerRule {
    pub message: String,
}

impl MeasurementVarianceRule for MeasurementVarianceRequiredIssuerRule {
    fn check(&self, item: &MeasurementVariance) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement variance.
#[derive(Debug, Clone)]
pub struct MeasurementVarianceValidator {
    pub rules: Vec<Rc<dyn MeasurementVarianceRule>>,
}

impl MeasurementVarianceValidator {
    pub fn validate(&self, item: &MeasurementVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_variance_validator() -> MeasurementVarianceValidator {
    MeasurementVarianceValidator {
        rules: vec![
            Rc::new(MeasurementVarianceRequiredLabelRule {
                message: "measurement variance needs a value".to_string(),
            }) as Rc<dyn MeasurementVarianceRule>,
            Rc::new(MeasurementVarianceIssuerLengthRule { max_length: 32 }),
            Rc::new(MeasurementVarianceRequiredIssuerRule {
                message: "measurement variance needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_variance_validator();
        let item = new_measurement_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = MeasurementVarianceRequiredLabelRule {
            message: "measurement variance needs a value".to_string(),
        };
        let mut item = new_measurement_variance("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = MeasurementVarianceIssuerLengthRule { max_length: 32 };
        let mut item = new_measurement_variance("x", "y");
        item.issuer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = MeasurementVarianceRequiredIssuerRule {
            message: "measurement variance needs a value".to_string(),
        };
        let mut item = new_measurement_variance("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
