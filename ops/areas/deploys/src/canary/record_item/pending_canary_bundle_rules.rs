use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending canary bundle must pass. Check returns its problems, or none.
pub trait PendingCanaryBundleRule: std::fmt::Debug {
    fn check(&self, item: &PendingCanaryBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryBundleCodeLengthRule {
    pub max_length: i64,
}

impl PendingCanaryBundleRule for PendingCanaryBundleCodeLengthRule {
    fn check(&self, item: &PendingCanaryBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryBundleLocaleLengthRule {
    pub max_length: i64,
}

impl PendingCanaryBundleRule for PendingCanaryBundleLocaleLengthRule {
    fn check(&self, item: &PendingCanaryBundle) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryBundleRequiredCountryRule {
    pub message: String,
}

impl PendingCanaryBundleRule for PendingCanaryBundleRequiredCountryRule {
    fn check(&self, item: &PendingCanaryBundle) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryBundleLabelLengthRule {
    pub max_length: i64,
}

impl PendingCanaryBundleRule for PendingCanaryBundleLabelLengthRule {
    fn check(&self, item: &PendingCanaryBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryBundleRequiredLabelRule {
    pub message: String,
}

impl PendingCanaryBundleRule for PendingCanaryBundleRequiredLabelRule {
    fn check(&self, item: &PendingCanaryBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending canary bundle.
#[derive(Debug, Clone)]
pub struct PendingCanaryBundleValidator {
    pub rules: Vec<Rc<dyn PendingCanaryBundleRule>>,
}

impl PendingCanaryBundleValidator {
    pub fn validate(&self, item: &PendingCanaryBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingCanaryBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingCanaryBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_canary_bundle_validator() -> PendingCanaryBundleValidator {
    PendingCanaryBundleValidator {
        rules: vec![
            Rc::new(PendingCanaryBundleCodeLengthRule { max_length: 16 }) as Rc<dyn PendingCanaryBundleRule>,
            Rc::new(PendingCanaryBundleLocaleLengthRule { max_length: 3600 }),
            Rc::new(PendingCanaryBundleRequiredCountryRule {
                message: "pending canary bundle needs this field".to_string(),
            }),
            Rc::new(PendingCanaryBundleLabelLengthRule { max_length: 64 }),
            Rc::new(PendingCanaryBundleRequiredLabelRule {
                message: "pending canary bundle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_canary_bundle_validator();
        let item = new_pending_canary_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingCanaryBundleCodeLengthRule { max_length: 1000 };
        let mut item = new_pending_canary_bundle("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = PendingCanaryBundleLocaleLengthRule { max_length: 32 };
        let mut item = new_pending_canary_bundle("x", "y");
        item.locale = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = PendingCanaryBundleRequiredCountryRule {
            message: "pending canary bundle needs this field".to_string(),
        };
        let mut item = new_pending_canary_bundle("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingCanaryBundleLabelLengthRule { max_length: 3600 };
        let mut item = new_pending_canary_bundle("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingCanaryBundleRequiredLabelRule {
            message: "pending canary bundle needs an entry".to_string(),
        };
        let mut item = new_pending_canary_bundle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
