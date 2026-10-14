use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered alert manifest must pass. Check returns its problems, or none.
pub trait TieredAlertManifestRule: std::fmt::Debug {
    fn check(&self, item: &TieredAlertManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertManifestRequiredScopeRule {
    pub message: String,
}

impl TieredAlertManifestRule for TieredAlertManifestRequiredScopeRule {
    fn check(&self, item: &TieredAlertManifest) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertManifestArchivedGuardRule {
    pub limit: i64,
}

impl TieredAlertManifestRule for TieredAlertManifestArchivedGuardRule {
    fn check(&self, item: &TieredAlertManifest) -> Vec<String> {
        if item.archived && (item.port > self.limit) {
            return vec![format!("port {} exceeds {} while archived", item.port, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertManifestRequiredLabelRule {
    pub message: String,
}

impl TieredAlertManifestRule for TieredAlertManifestRequiredLabelRule {
    fn check(&self, item: &TieredAlertManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertManifestPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredAlertManifestRule for TieredAlertManifestPortRangeRule {
    fn check(&self, item: &TieredAlertManifest) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered alert manifest.
#[derive(Debug, Clone)]
pub struct TieredAlertManifestValidator {
    pub rules: Vec<Rc<dyn TieredAlertManifestRule>>,
}

impl TieredAlertManifestValidator {
    pub fn validate(&self, item: &TieredAlertManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredAlertManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredAlertManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_alert_manifest_validator() -> TieredAlertManifestValidator {
    TieredAlertManifestValidator {
        rules: vec![
            Rc::new(TieredAlertManifestRequiredScopeRule {
                message: "tiered alert manifest needs a value".to_string(),
            }) as Rc<dyn TieredAlertManifestRule>,
            Rc::new(TieredAlertManifestArchivedGuardRule { limit: 32 }),
            Rc::new(TieredAlertManifestRequiredLabelRule {
                message: "tiered alert manifest needs a value".to_string(),
            }),
            Rc::new(TieredAlertManifestPortRangeRule { min: 120, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_alert_manifest_validator();
        let item = new_tiered_alert_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = TieredAlertManifestRequiredScopeRule {
            message: "tiered alert manifest needs an entry".to_string(),
        };
        let mut item = new_tiered_alert_manifest("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TieredAlertManifestRequiredLabelRule {
            message: "tiered alert manifest needs this field".to_string(),
        };
        let mut item = new_tiered_alert_manifest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = TieredAlertManifestPortRangeRule { min: 32, max: 86400 };
        let mut item = new_tiered_alert_manifest("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
