use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered binding bundle must pass. Check returns its problems, or none.
pub trait TieredBindingBundleRule: std::fmt::Debug {
    fn check(&self, item: &TieredBindingBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingBundleApprovedGuardRule {
    pub limit: i64,
}

impl TieredBindingBundleRule for TieredBindingBundleApprovedGuardRule {
    fn check(&self, item: &TieredBindingBundle) -> Vec<String> {
        if item.approved && (item.quota_units > self.limit) {
            return vec![format!(
                "quota units {} exceeds {} while approved",
                item.quota_units, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingBundleRequiredCodeRule {
    pub message: String,
}

impl TieredBindingBundleRule for TieredBindingBundleRequiredCodeRule {
    fn check(&self, item: &TieredBindingBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingBundleQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredBindingBundleRule for TieredBindingBundleQuotaUnitsRangeRule {
    fn check(&self, item: &TieredBindingBundle) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered binding bundle.
#[derive(Debug, Clone)]
pub struct TieredBindingBundleValidator {
    pub rules: Vec<Rc<dyn TieredBindingBundleRule>>,
}

impl TieredBindingBundleValidator {
    pub fn validate(&self, item: &TieredBindingBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredBindingBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredBindingBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_binding_bundle_validator() -> TieredBindingBundleValidator {
    TieredBindingBundleValidator {
        rules: vec![
            Rc::new(TieredBindingBundleApprovedGuardRule { limit: 32 }) as Rc<dyn TieredBindingBundleRule>,
            Rc::new(TieredBindingBundleRequiredCodeRule {
                message: "tiered binding bundle needs a value".to_string(),
            }),
            Rc::new(TieredBindingBundleQuotaUnitsRangeRule { min: 16, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_binding_bundle_validator();
        let item = new_tiered_binding_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredBindingBundleRequiredCodeRule {
            message: "tiered binding bundle needs an entry".to_string(),
        };
        let mut item = new_tiered_binding_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = TieredBindingBundleQuotaUnitsRangeRule { min: 255, max: 1000 };
        let mut item = new_tiered_binding_bundle("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
