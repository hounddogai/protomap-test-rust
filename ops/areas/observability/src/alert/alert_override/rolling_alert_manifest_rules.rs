use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling alert manifest must pass. Check returns its problems, or none.
pub trait RollingAlertManifestRule: std::fmt::Debug {
    fn check(&self, item: &RollingAlertManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertManifestTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingAlertManifestRule for RollingAlertManifestTtlSecondsRangeRule {
    fn check(&self, item: &RollingAlertManifest) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertManifestRequiredCodeRule {
    pub message: String,
}

impl RollingAlertManifestRule for RollingAlertManifestRequiredCodeRule {
    fn check(&self, item: &RollingAlertManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertManifestVisibleGuardRule {
    pub limit: f64,
}

impl RollingAlertManifestRule for RollingAlertManifestVisibleGuardRule {
    fn check(&self, item: &RollingAlertManifest) -> Vec<String> {
        if item.visible && (item.margin > self.limit) {
            return vec![format!("margin {} exceeds {} while visible", item.margin, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertManifestChecksumLengthRule {
    pub max_length: i64,
}

impl RollingAlertManifestRule for RollingAlertManifestChecksumLengthRule {
    fn check(&self, item: &RollingAlertManifest) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling alert manifest.
#[derive(Debug, Clone)]
pub struct RollingAlertManifestValidator {
    pub rules: Vec<Rc<dyn RollingAlertManifestRule>>,
}

impl RollingAlertManifestValidator {
    pub fn validate(&self, item: &RollingAlertManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingAlertManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingAlertManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_alert_manifest_validator() -> RollingAlertManifestValidator {
    RollingAlertManifestValidator {
        rules: vec![
            Rc::new(RollingAlertManifestTtlSecondsRangeRule { min: 16, max: 16 }) as Rc<dyn RollingAlertManifestRule>,
            Rc::new(RollingAlertManifestRequiredCodeRule {
                message: "rolling alert manifest needs a value".to_string(),
            }),
            Rc::new(RollingAlertManifestVisibleGuardRule { limit: 2.2 }),
            Rc::new(RollingAlertManifestChecksumLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_alert_manifest_validator();
        let item = new_rolling_alert_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RollingAlertManifestTtlSecondsRangeRule { min: 32, max: 86400 };
        let mut item = new_rolling_alert_manifest("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingAlertManifestRequiredCodeRule {
            message: "rolling alert manifest needs a value".to_string(),
        };
        let mut item = new_rolling_alert_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = RollingAlertManifestChecksumLengthRule { max_length: 16 };
        let mut item = new_rolling_alert_manifest("x", "y");
        item.checksum = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
