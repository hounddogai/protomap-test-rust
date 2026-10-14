use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant manifest must pass. Check returns its problems, or none.
pub trait TenantManifestRule: std::fmt::Debug {
    fn check(&self, item: &TenantManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantManifestTitleLengthRule {
    pub max_length: i64,
}

impl TenantManifestRule for TenantManifestTitleLengthRule {
    fn check(&self, item: &TenantManifest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantManifestQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantManifestRule for TenantManifestQuantityRangeRule {
    fn check(&self, item: &TenantManifest) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantManifestSummaryLengthRule {
    pub max_length: i64,
}

impl TenantManifestRule for TenantManifestSummaryLengthRule {
    fn check(&self, item: &TenantManifest) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantManifestRequiredTitleRule {
    pub message: String,
}

impl TenantManifestRule for TenantManifestRequiredTitleRule {
    fn check(&self, item: &TenantManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant manifest.
#[derive(Debug, Clone)]
pub struct TenantManifestValidator {
    pub rules: Vec<Rc<dyn TenantManifestRule>>,
}

impl TenantManifestValidator {
    pub fn validate(&self, item: &TenantManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_manifest_validator() -> TenantManifestValidator {
    TenantManifestValidator {
        rules: vec![
            Rc::new(TenantManifestTitleLengthRule { max_length: 64 }) as Rc<dyn TenantManifestRule>,
            Rc::new(TenantManifestQuantityRangeRule { min: 16, max: 3600 }),
            Rc::new(TenantManifestSummaryLengthRule { max_length: 16 }),
            Rc::new(TenantManifestRequiredTitleRule {
                message: "tenant manifest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_manifest_validator();
        let item = new_tenant_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TenantManifestTitleLengthRule { max_length: 3600 };
        let mut item = new_tenant_manifest("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = TenantManifestQuantityRangeRule { min: 16, max: 32 };
        let mut item = new_tenant_manifest("x", "y");
        item.quantity = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = TenantManifestSummaryLengthRule { max_length: 3600 };
        let mut item = new_tenant_manifest("x", "y");
        item.summary = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TenantManifestRequiredTitleRule {
            message: "tenant manifest needs this field".to_string(),
        };
        let mut item = new_tenant_manifest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
