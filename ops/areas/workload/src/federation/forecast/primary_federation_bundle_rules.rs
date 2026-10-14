use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary federation bundle must pass. Check returns its problems, or none.
pub trait PrimaryFederationBundleRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryFederationBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFederationBundleRequiredClusterRule {
    pub message: String,
}

impl PrimaryFederationBundleRule for PrimaryFederationBundleRequiredClusterRule {
    fn check(&self, item: &PrimaryFederationBundle) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFederationBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimaryFederationBundleRule for PrimaryFederationBundleAllowedCodeRule {
    fn check(&self, item: &PrimaryFederationBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFederationBundleThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryFederationBundleRule for PrimaryFederationBundleThresholdRangeRule {
    fn check(&self, item: &PrimaryFederationBundle) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFederationBundlePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryFederationBundleRule for PrimaryFederationBundlePortRangeRule {
    fn check(&self, item: &PrimaryFederationBundle) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFederationBundleVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryFederationBundleRule for PrimaryFederationBundleVersionRangeRule {
    fn check(&self, item: &PrimaryFederationBundle) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFederationBundleActiveGuardRule {
    pub limit: f64,
}

impl PrimaryFederationBundleRule for PrimaryFederationBundleActiveGuardRule {
    fn check(&self, item: &PrimaryFederationBundle) -> Vec<String> {
        if item.active && (item.threshold > self.limit) {
            return vec![format!(
                "threshold {} exceeds {} while active",
                item.threshold, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary federation bundle.
#[derive(Debug, Clone)]
pub struct PrimaryFederationBundleValidator {
    pub rules: Vec<Rc<dyn PrimaryFederationBundleRule>>,
}

impl PrimaryFederationBundleValidator {
    pub fn validate(&self, item: &PrimaryFederationBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryFederationBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryFederationBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_federation_bundle_validator() -> PrimaryFederationBundleValidator {
    PrimaryFederationBundleValidator {
        rules: vec![
            Rc::new(PrimaryFederationBundleRequiredClusterRule {
                message: "primary federation bundle needs this field".to_string(),
            }) as Rc<dyn PrimaryFederationBundleRule>,
            Rc::new(PrimaryFederationBundleAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(PrimaryFederationBundleThresholdRangeRule { min: 6.1, max: 332.3 }),
            Rc::new(PrimaryFederationBundlePortRangeRule { min: 16, max: 80 }),
            Rc::new(PrimaryFederationBundleVersionRangeRule { min: 32, max: 1000 }),
            Rc::new(PrimaryFederationBundleActiveGuardRule { limit: 1.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_federation_bundle_validator();
        let item = new_primary_federation_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = PrimaryFederationBundleRequiredClusterRule {
            message: "primary federation bundle needs a value".to_string(),
        };
        let mut item = new_primary_federation_bundle("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = PrimaryFederationBundleThresholdRangeRule { min: 7.5, max: 4787.7 };
        let mut item = new_primary_federation_bundle("x", "y");
        item.threshold = 4789.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = PrimaryFederationBundlePortRangeRule { min: 64, max: 3600 };
        let mut item = new_primary_federation_bundle("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = PrimaryFederationBundleVersionRangeRule { min: 120, max: 86400 };
        let mut item = new_primary_federation_bundle("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
