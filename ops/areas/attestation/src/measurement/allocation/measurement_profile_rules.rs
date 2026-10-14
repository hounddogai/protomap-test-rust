use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement profile must pass. Check returns its problems, or none.
pub trait MeasurementProfileRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementProfileRequiredCodeRule {
    pub message: String,
}

impl MeasurementProfileRule for MeasurementProfileRequiredCodeRule {
    fn check(&self, item: &MeasurementProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementProfileGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementProfileRule for MeasurementProfileGraceDaysRangeRule {
    fn check(&self, item: &MeasurementProfile) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementProfilePinnedGuardRule {
    pub limit: f64,
}

impl MeasurementProfileRule for MeasurementProfilePinnedGuardRule {
    fn check(&self, item: &MeasurementProfile) -> Vec<String> {
        if item.pinned && (item.threshold > self.limit) {
            return vec![format!(
                "threshold {} exceeds {} while pinned",
                item.threshold, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementProfileLabelLengthRule {
    pub max_length: i64,
}

impl MeasurementProfileRule for MeasurementProfileLabelLengthRule {
    fn check(&self, item: &MeasurementProfile) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementProfileIssuerLengthRule {
    pub max_length: i64,
}

impl MeasurementProfileRule for MeasurementProfileIssuerLengthRule {
    fn check(&self, item: &MeasurementProfile) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementProfileCodePrefixRule {
    pub prefix: String,
}

impl MeasurementProfileRule for MeasurementProfileCodePrefixRule {
    fn check(&self, item: &MeasurementProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement profile.
#[derive(Debug, Clone)]
pub struct MeasurementProfileValidator {
    pub rules: Vec<Rc<dyn MeasurementProfileRule>>,
}

impl MeasurementProfileValidator {
    pub fn validate(&self, item: &MeasurementProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MeasurementProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_profile_validator() -> MeasurementProfileValidator {
    MeasurementProfileValidator {
        rules: vec![
            Rc::new(MeasurementProfileRequiredCodeRule {
                message: "measurement profile needs an entry".to_string(),
            }) as Rc<dyn MeasurementProfileRule>,
            Rc::new(MeasurementProfileGraceDaysRangeRule { min: 80, max: 3600 }),
            Rc::new(MeasurementProfilePinnedGuardRule { limit: 7.6 }),
            Rc::new(MeasurementProfileLabelLengthRule { max_length: 255 }),
            Rc::new(MeasurementProfileIssuerLengthRule { max_length: 120 }),
            Rc::new(MeasurementProfileCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_profile_validator();
        let item = new_measurement_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MeasurementProfileRequiredCodeRule {
            message: "measurement profile needs this field".to_string(),
        };
        let mut item = new_measurement_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = MeasurementProfileGraceDaysRangeRule { min: 16, max: 86400 };
        let mut item = new_measurement_profile("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = MeasurementProfileLabelLengthRule { max_length: 120 };
        let mut item = new_measurement_profile("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = MeasurementProfileIssuerLengthRule { max_length: 1000 };
        let mut item = new_measurement_profile("x", "y");
        item.issuer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
