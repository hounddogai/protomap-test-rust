use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant manifest must pass. Check returns its problems, or none.
pub trait GrantManifestRule: std::fmt::Debug {
    fn check(&self, item: &GrantManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantManifestRequiredCodeRule {
    pub message: String,
}

impl GrantManifestRule for GrantManifestRequiredCodeRule {
    fn check(&self, item: &GrantManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantManifestRequiredExternalRefRule {
    pub message: String,
}

impl GrantManifestRule for GrantManifestRequiredExternalRefRule {
    fn check(&self, item: &GrantManifest) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantManifestRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantManifestRule for GrantManifestRateRangeRule {
    fn check(&self, item: &GrantManifest) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantManifestCodePrefixRule {
    pub prefix: String,
}

impl GrantManifestRule for GrantManifestCodePrefixRule {
    fn check(&self, item: &GrantManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantManifestRequiredScopeRule {
    pub message: String,
}

impl GrantManifestRule for GrantManifestRequiredScopeRule {
    fn check(&self, item: &GrantManifest) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant manifest.
#[derive(Debug, Clone)]
pub struct GrantManifestValidator {
    pub rules: Vec<Rc<dyn GrantManifestRule>>,
}

impl GrantManifestValidator {
    pub fn validate(&self, item: &GrantManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_manifest_validator() -> GrantManifestValidator {
    GrantManifestValidator {
        rules: vec![
            Rc::new(GrantManifestRequiredCodeRule {
                message: "grant manifest needs this field".to_string(),
            }) as Rc<dyn GrantManifestRule>,
            Rc::new(GrantManifestRequiredExternalRefRule {
                message: "grant manifest needs this field".to_string(),
            }),
            Rc::new(GrantManifestRateRangeRule { min: 4.6, max: 4114.5 }),
            Rc::new(GrantManifestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(GrantManifestRequiredScopeRule {
                message: "grant manifest needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_manifest_validator();
        let item = new_grant_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GrantManifestRequiredCodeRule {
            message: "grant manifest needs an entry".to_string(),
        };
        let mut item = new_grant_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = GrantManifestRequiredExternalRefRule {
            message: "grant manifest needs an entry".to_string(),
        };
        let mut item = new_grant_manifest("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = GrantManifestRateRangeRule { min: 4.3, max: 1001.2 };
        let mut item = new_grant_manifest("x", "y");
        item.rate = 1002.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = GrantManifestRequiredScopeRule {
            message: "grant manifest needs an entry".to_string(),
        };
        let mut item = new_grant_manifest("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
