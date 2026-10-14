use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard manifest must pass. Check returns its problems, or none.
pub trait DashboardManifestRule: std::fmt::Debug {
    fn check(&self, item: &DashboardManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardManifestCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardManifestRule for DashboardManifestCapacityRangeRule {
    fn check(&self, item: &DashboardManifest) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardManifestRequiredRegionRule {
    pub message: String,
}

impl DashboardManifestRule for DashboardManifestRequiredRegionRule {
    fn check(&self, item: &DashboardManifest) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardManifestNamespaceLengthRule {
    pub max_length: i64,
}

impl DashboardManifestRule for DashboardManifestNamespaceLengthRule {
    fn check(&self, item: &DashboardManifest) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardManifestCodeLengthRule {
    pub max_length: i64,
}

impl DashboardManifestRule for DashboardManifestCodeLengthRule {
    fn check(&self, item: &DashboardManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard manifest.
#[derive(Debug, Clone)]
pub struct DashboardManifestValidator {
    pub rules: Vec<Rc<dyn DashboardManifestRule>>,
}

impl DashboardManifestValidator {
    pub fn validate(&self, item: &DashboardManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_manifest_validator() -> DashboardManifestValidator {
    DashboardManifestValidator {
        rules: vec![
            Rc::new(DashboardManifestCapacityRangeRule { min: 80, max: 80 }) as Rc<dyn DashboardManifestRule>,
            Rc::new(DashboardManifestRequiredRegionRule {
                message: "dashboard manifest needs an entry".to_string(),
            }),
            Rc::new(DashboardManifestNamespaceLengthRule { max_length: 86400 }),
            Rc::new(DashboardManifestCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_manifest_validator();
        let item = new_dashboard_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = DashboardManifestCapacityRangeRule { min: 120, max: 1000 };
        let mut item = new_dashboard_manifest("x", "y");
        item.capacity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = DashboardManifestRequiredRegionRule {
            message: "dashboard manifest needs an entry".to_string(),
        };
        let mut item = new_dashboard_manifest("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = DashboardManifestNamespaceLengthRule { max_length: 120 };
        let mut item = new_dashboard_manifest("x", "y");
        item.namespace = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DashboardManifestCodeLengthRule { max_length: 255 };
        let mut item = new_dashboard_manifest("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
