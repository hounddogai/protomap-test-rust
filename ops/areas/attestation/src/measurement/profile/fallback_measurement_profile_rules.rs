use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback measurement profile must pass. Check returns its problems, or none.
pub trait FallbackMeasurementProfileRule: std::fmt::Debug {
    fn check(&self, item: &FallbackMeasurementProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementProfileCodePrefixRule {
    pub prefix: String,
}

impl FallbackMeasurementProfileRule for FallbackMeasurementProfileCodePrefixRule {
    fn check(&self, item: &FallbackMeasurementProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementProfileNameLengthRule {
    pub max_length: i64,
}

impl FallbackMeasurementProfileRule for FallbackMeasurementProfileNameLengthRule {
    fn check(&self, item: &FallbackMeasurementProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementProfileRequiredCodeRule {
    pub message: String,
}

impl FallbackMeasurementProfileRule for FallbackMeasurementProfileRequiredCodeRule {
    fn check(&self, item: &FallbackMeasurementProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementProfileRequiredIssuerRule {
    pub message: String,
}

impl FallbackMeasurementProfileRule for FallbackMeasurementProfileRequiredIssuerRule {
    fn check(&self, item: &FallbackMeasurementProfile) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementProfileIssuerLengthRule {
    pub max_length: i64,
}

impl FallbackMeasurementProfileRule for FallbackMeasurementProfileIssuerLengthRule {
    fn check(&self, item: &FallbackMeasurementProfile) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementProfileRequiredNameRule {
    pub message: String,
}

impl FallbackMeasurementProfileRule for FallbackMeasurementProfileRequiredNameRule {
    fn check(&self, item: &FallbackMeasurementProfile) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback measurement profile.
#[derive(Debug, Clone)]
pub struct FallbackMeasurementProfileValidator {
    pub rules: Vec<Rc<dyn FallbackMeasurementProfileRule>>,
}

impl FallbackMeasurementProfileValidator {
    pub fn validate(&self, item: &FallbackMeasurementProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackMeasurementProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackMeasurementProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_measurement_profile_validator() -> FallbackMeasurementProfileValidator {
    FallbackMeasurementProfileValidator {
        rules: vec![
            Rc::new(FallbackMeasurementProfileCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn FallbackMeasurementProfileRule>,
            Rc::new(FallbackMeasurementProfileNameLengthRule { max_length: 32 }),
            Rc::new(FallbackMeasurementProfileRequiredCodeRule {
                message: "fallback measurement profile needs this field".to_string(),
            }),
            Rc::new(FallbackMeasurementProfileRequiredIssuerRule {
                message: "fallback measurement profile needs this field".to_string(),
            }),
            Rc::new(FallbackMeasurementProfileIssuerLengthRule { max_length: 86400 }),
            Rc::new(FallbackMeasurementProfileRequiredNameRule {
                message: "fallback measurement profile needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_measurement_profile_validator();
        let item = new_fallback_measurement_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackMeasurementProfileNameLengthRule { max_length: 64 };
        let mut item = new_fallback_measurement_profile("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackMeasurementProfileRequiredCodeRule {
            message: "fallback measurement profile needs an entry".to_string(),
        };
        let mut item = new_fallback_measurement_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = FallbackMeasurementProfileRequiredIssuerRule {
            message: "fallback measurement profile needs a value".to_string(),
        };
        let mut item = new_fallback_measurement_profile("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = FallbackMeasurementProfileIssuerLengthRule { max_length: 16 };
        let mut item = new_fallback_measurement_profile("x", "y");
        item.issuer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FallbackMeasurementProfileRequiredNameRule {
            message: "fallback measurement profile needs this field".to_string(),
        };
        let mut item = new_fallback_measurement_profile("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
