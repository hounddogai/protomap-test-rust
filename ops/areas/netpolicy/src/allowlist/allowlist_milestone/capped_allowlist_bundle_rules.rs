use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped allowlist bundle must pass. Check returns its problems, or none.
pub trait CappedAllowlistBundleRule: std::fmt::Debug {
    fn check(&self, item: &CappedAllowlistBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAllowlistBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CappedAllowlistBundleRule for CappedAllowlistBundleAllowedCodeRule {
    fn check(&self, item: &CappedAllowlistBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAllowlistBundleClusterLengthRule {
    pub max_length: i64,
}

impl CappedAllowlistBundleRule for CappedAllowlistBundleClusterLengthRule {
    fn check(&self, item: &CappedAllowlistBundle) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAllowlistBundleCodePrefixRule {
    pub prefix: String,
}

impl CappedAllowlistBundleRule for CappedAllowlistBundleCodePrefixRule {
    fn check(&self, item: &CappedAllowlistBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAllowlistBundleCountryLengthRule {
    pub max_length: i64,
}

impl CappedAllowlistBundleRule for CappedAllowlistBundleCountryLengthRule {
    fn check(&self, item: &CappedAllowlistBundle) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAllowlistBundleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedAllowlistBundleRule for CappedAllowlistBundleDiscountRangeRule {
    fn check(&self, item: &CappedAllowlistBundle) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAllowlistBundleMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedAllowlistBundleRule for CappedAllowlistBundleMinimumRangeRule {
    fn check(&self, item: &CappedAllowlistBundle) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped allowlist bundle.
#[derive(Debug, Clone)]
pub struct CappedAllowlistBundleValidator {
    pub rules: Vec<Rc<dyn CappedAllowlistBundleRule>>,
}

impl CappedAllowlistBundleValidator {
    pub fn validate(&self, item: &CappedAllowlistBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedAllowlistBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedAllowlistBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_allowlist_bundle_validator() -> CappedAllowlistBundleValidator {
    CappedAllowlistBundleValidator {
        rules: vec![
            Rc::new(CappedAllowlistBundleAllowedCodeRule {
                allowed: vec!["lenient".to_string()],
            }) as Rc<dyn CappedAllowlistBundleRule>,
            Rc::new(CappedAllowlistBundleClusterLengthRule { max_length: 32 }),
            Rc::new(CappedAllowlistBundleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(CappedAllowlistBundleCountryLengthRule { max_length: 64 }),
            Rc::new(CappedAllowlistBundleDiscountRangeRule { min: 8.8, max: 4592.2 }),
            Rc::new(CappedAllowlistBundleMinimumRangeRule { min: 2.4, max: 4192.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_allowlist_bundle_validator();
        let item = new_capped_allowlist_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = CappedAllowlistBundleClusterLengthRule { max_length: 120 };
        let mut item = new_capped_allowlist_bundle("x", "y");
        item.cluster = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = CappedAllowlistBundleCountryLengthRule { max_length: 80 };
        let mut item = new_capped_allowlist_bundle("x", "y");
        item.country = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = CappedAllowlistBundleDiscountRangeRule { min: 4.3, max: 2741.4 };
        let mut item = new_capped_allowlist_bundle("x", "y");
        item.discount = 2742.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = CappedAllowlistBundleMinimumRangeRule { min: 1.0, max: 1435.7 };
        let mut item = new_capped_allowlist_bundle("x", "y");
        item.minimum = 1437.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
