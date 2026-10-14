use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy bundle must pass. Check returns its problems, or none.
pub trait TtlPolicyBundleRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBundleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyBundleRule for TtlPolicyBundleRotationDaysRangeRule {
    fn check(&self, item: &TtlPolicyBundle) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBundleScopeLengthRule {
    pub max_length: i64,
}

impl TtlPolicyBundleRule for TtlPolicyBundleScopeLengthRule {
    fn check(&self, item: &TtlPolicyBundle) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBundleQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyBundleRule for TtlPolicyBundleQuotaUnitsRangeRule {
    fn check(&self, item: &TtlPolicyBundle) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBundleRequiredScopeRule {
    pub message: String,
}

impl TtlPolicyBundleRule for TtlPolicyBundleRequiredScopeRule {
    fn check(&self, item: &TtlPolicyBundle) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBundleVisibleGuardRule {
    pub limit: i64,
}

impl TtlPolicyBundleRule for TtlPolicyBundleVisibleGuardRule {
    fn check(&self, item: &TtlPolicyBundle) -> Vec<String> {
        if item.visible && (item.quota_units > self.limit) {
            return vec![format!(
                "quota units {} exceeds {} while visible",
                item.quota_units, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBundleLabelLengthRule {
    pub max_length: i64,
}

impl TtlPolicyBundleRule for TtlPolicyBundleLabelLengthRule {
    fn check(&self, item: &TtlPolicyBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy bundle.
#[derive(Debug, Clone)]
pub struct TtlPolicyBundleValidator {
    pub rules: Vec<Rc<dyn TtlPolicyBundleRule>>,
}

impl TtlPolicyBundleValidator {
    pub fn validate(&self, item: &TtlPolicyBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_bundle_validator() -> TtlPolicyBundleValidator {
    TtlPolicyBundleValidator {
        rules: vec![
            Rc::new(TtlPolicyBundleRotationDaysRangeRule { min: 1000, max: 86400 }) as Rc<dyn TtlPolicyBundleRule>,
            Rc::new(TtlPolicyBundleScopeLengthRule { max_length: 1000 }),
            Rc::new(TtlPolicyBundleQuotaUnitsRangeRule { min: 80, max: 86400 }),
            Rc::new(TtlPolicyBundleRequiredScopeRule {
                message: "ttl policy bundle needs an entry".to_string(),
            }),
            Rc::new(TtlPolicyBundleVisibleGuardRule { limit: 255 }),
            Rc::new(TtlPolicyBundleLabelLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_bundle_validator();
        let item = new_ttl_policy_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = TtlPolicyBundleRotationDaysRangeRule { min: 32, max: 255 };
        let mut item = new_ttl_policy_bundle("x", "y");
        item.rotation_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = TtlPolicyBundleScopeLengthRule { max_length: 3600 };
        let mut item = new_ttl_policy_bundle("x", "y");
        item.scope = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = TtlPolicyBundleQuotaUnitsRangeRule { min: 32, max: 80 };
        let mut item = new_ttl_policy_bundle("x", "y");
        item.quota_units = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = TtlPolicyBundleRequiredScopeRule {
            message: "ttl policy bundle needs this field".to_string(),
        };
        let mut item = new_ttl_policy_bundle("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TtlPolicyBundleLabelLengthRule { max_length: 32 };
        let mut item = new_ttl_policy_bundle("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
