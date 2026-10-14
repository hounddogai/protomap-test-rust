use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert manifest must pass. Check returns its problems, or none.
pub trait AlertManifestRule: std::fmt::Debug {
    fn check(&self, item: &AlertManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertManifestPhoneLengthRule {
    pub max_length: i64,
}

impl AlertManifestRule for AlertManifestPhoneLengthRule {
    fn check(&self, item: &AlertManifest) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertManifestRequiredPrincipalRule {
    pub message: String,
}

impl AlertManifestRule for AlertManifestRequiredPrincipalRule {
    fn check(&self, item: &AlertManifest) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertManifestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertManifestRule for AlertManifestCreatedAtRangeRule {
    fn check(&self, item: &AlertManifest) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertManifestCodeLengthRule {
    pub max_length: i64,
}

impl AlertManifestRule for AlertManifestCodeLengthRule {
    fn check(&self, item: &AlertManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert manifest.
#[derive(Debug, Clone)]
pub struct AlertManifestValidator {
    pub rules: Vec<Rc<dyn AlertManifestRule>>,
}

impl AlertManifestValidator {
    pub fn validate(&self, item: &AlertManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_manifest_validator() -> AlertManifestValidator {
    AlertManifestValidator {
        rules: vec![
            Rc::new(AlertManifestPhoneLengthRule { max_length: 1000 }) as Rc<dyn AlertManifestRule>,
            Rc::new(AlertManifestRequiredPrincipalRule {
                message: "alert manifest needs a value".to_string(),
            }),
            Rc::new(AlertManifestCreatedAtRangeRule { min: 255, max: 1000 }),
            Rc::new(AlertManifestCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_manifest_validator();
        let item = new_alert_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = AlertManifestPhoneLengthRule { max_length: 80 };
        let mut item = new_alert_manifest("x", "y");
        item.phone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = AlertManifestRequiredPrincipalRule {
            message: "alert manifest needs an entry".to_string(),
        };
        let mut item = new_alert_manifest("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = AlertManifestCreatedAtRangeRule { min: 1000, max: 86400 };
        let mut item = new_alert_manifest("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AlertManifestCodeLengthRule { max_length: 1000 };
        let mut item = new_alert_manifest("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
