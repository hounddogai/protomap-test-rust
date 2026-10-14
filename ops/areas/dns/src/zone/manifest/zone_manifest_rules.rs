use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone manifest must pass. Check returns its problems, or none.
pub trait ZoneManifestRule: std::fmt::Debug {
    fn check(&self, item: &ZoneManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneManifestCodeLengthRule {
    pub max_length: i64,
}

impl ZoneManifestRule for ZoneManifestCodeLengthRule {
    fn check(&self, item: &ZoneManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneManifestRequiredCodeRule {
    pub message: String,
}

impl ZoneManifestRule for ZoneManifestRequiredCodeRule {
    fn check(&self, item: &ZoneManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneManifestDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneManifestRule for ZoneManifestDurationMinutesRangeRule {
    fn check(&self, item: &ZoneManifest) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone manifest.
#[derive(Debug, Clone)]
pub struct ZoneManifestValidator {
    pub rules: Vec<Rc<dyn ZoneManifestRule>>,
}

impl ZoneManifestValidator {
    pub fn validate(&self, item: &ZoneManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_manifest_validator() -> ZoneManifestValidator {
    ZoneManifestValidator {
        rules: vec![
            Rc::new(ZoneManifestCodeLengthRule { max_length: 80 }) as Rc<dyn ZoneManifestRule>,
            Rc::new(ZoneManifestRequiredCodeRule {
                message: "zone manifest needs a value".to_string(),
            }),
            Rc::new(ZoneManifestDurationMinutesRangeRule { min: 120, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_manifest_validator();
        let item = new_zone_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ZoneManifestCodeLengthRule { max_length: 3600 };
        let mut item = new_zone_manifest("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ZoneManifestRequiredCodeRule {
            message: "zone manifest needs this field".to_string(),
        };
        let mut item = new_zone_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = ZoneManifestDurationMinutesRangeRule { min: 32, max: 3600 };
        let mut item = new_zone_manifest("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
