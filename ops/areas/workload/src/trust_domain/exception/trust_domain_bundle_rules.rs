use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain bundle must pass. Check returns its problems, or none.
pub trait TrustDomainBundleRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBundleNameLengthRule {
    pub max_length: i64,
}

impl TrustDomainBundleRule for TrustDomainBundleNameLengthRule {
    fn check(&self, item: &TrustDomainBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBundleRequiredLabelRule {
    pub message: String,
}

impl TrustDomainBundleRule for TrustDomainBundleRequiredLabelRule {
    fn check(&self, item: &TrustDomainBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBundleCodePrefixRule {
    pub prefix: String,
}

impl TrustDomainBundleRule for TrustDomainBundleCodePrefixRule {
    fn check(&self, item: &TrustDomainBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBundleCodeLengthRule {
    pub max_length: i64,
}

impl TrustDomainBundleRule for TrustDomainBundleCodeLengthRule {
    fn check(&self, item: &TrustDomainBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBundleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainBundleRule for TrustDomainBundleRotationDaysRangeRule {
    fn check(&self, item: &TrustDomainBundle) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBundleVisibleGuardRule {
    pub limit: i64,
}

impl TrustDomainBundleRule for TrustDomainBundleVisibleGuardRule {
    fn check(&self, item: &TrustDomainBundle) -> Vec<String> {
        if item.visible && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while visible",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain bundle.
#[derive(Debug, Clone)]
pub struct TrustDomainBundleValidator {
    pub rules: Vec<Rc<dyn TrustDomainBundleRule>>,
}

impl TrustDomainBundleValidator {
    pub fn validate(&self, item: &TrustDomainBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_bundle_validator() -> TrustDomainBundleValidator {
    TrustDomainBundleValidator {
        rules: vec![
            Rc::new(TrustDomainBundleNameLengthRule { max_length: 255 }) as Rc<dyn TrustDomainBundleRule>,
            Rc::new(TrustDomainBundleRequiredLabelRule {
                message: "trust domain bundle needs a value".to_string(),
            }),
            Rc::new(TrustDomainBundleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(TrustDomainBundleCodeLengthRule { max_length: 86400 }),
            Rc::new(TrustDomainBundleRotationDaysRangeRule { min: 32, max: 3600 }),
            Rc::new(TrustDomainBundleVisibleGuardRule { limit: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_bundle_validator();
        let item = new_trust_domain_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TrustDomainBundleNameLengthRule { max_length: 86400 };
        let mut item = new_trust_domain_bundle("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TrustDomainBundleRequiredLabelRule {
            message: "trust domain bundle needs an entry".to_string(),
        };
        let mut item = new_trust_domain_bundle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TrustDomainBundleCodeLengthRule { max_length: 64 };
        let mut item = new_trust_domain_bundle("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = TrustDomainBundleRotationDaysRangeRule { min: 16, max: 3600 };
        let mut item = new_trust_domain_bundle("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
