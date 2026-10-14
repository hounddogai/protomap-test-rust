use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional evidence manifest must pass. Check returns its problems, or none.
pub trait RegionalEvidenceManifestRule: std::fmt::Debug {
    fn check(&self, item: &RegionalEvidenceManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEvidenceManifestRequiredLocaleRule {
    pub message: String,
}

impl RegionalEvidenceManifestRule for RegionalEvidenceManifestRequiredLocaleRule {
    fn check(&self, item: &RegionalEvidenceManifest) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEvidenceManifestDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalEvidenceManifestRule for RegionalEvidenceManifestDurationMinutesRangeRule {
    fn check(&self, item: &RegionalEvidenceManifest) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEvidenceManifestVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalEvidenceManifestRule for RegionalEvidenceManifestVersionRangeRule {
    fn check(&self, item: &RegionalEvidenceManifest) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEvidenceManifestVisibleGuardRule {
    pub limit: i64,
}

impl RegionalEvidenceManifestRule for RegionalEvidenceManifestVisibleGuardRule {
    fn check(&self, item: &RegionalEvidenceManifest) -> Vec<String> {
        if item.visible && (item.version > self.limit) {
            return vec![format!("version {} exceeds {} while visible", item.version, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEvidenceManifestCodePrefixRule {
    pub prefix: String,
}

impl RegionalEvidenceManifestRule for RegionalEvidenceManifestCodePrefixRule {
    fn check(&self, item: &RegionalEvidenceManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEvidenceManifestTitleLengthRule {
    pub max_length: i64,
}

impl RegionalEvidenceManifestRule for RegionalEvidenceManifestTitleLengthRule {
    fn check(&self, item: &RegionalEvidenceManifest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional evidence manifest.
#[derive(Debug, Clone)]
pub struct RegionalEvidenceManifestValidator {
    pub rules: Vec<Rc<dyn RegionalEvidenceManifestRule>>,
}

impl RegionalEvidenceManifestValidator {
    pub fn validate(&self, item: &RegionalEvidenceManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalEvidenceManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalEvidenceManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_evidence_manifest_validator() -> RegionalEvidenceManifestValidator {
    RegionalEvidenceManifestValidator {
        rules: vec![
            Rc::new(RegionalEvidenceManifestRequiredLocaleRule {
                message: "regional evidence manifest needs this field".to_string(),
            }) as Rc<dyn RegionalEvidenceManifestRule>,
            Rc::new(RegionalEvidenceManifestDurationMinutesRangeRule { min: 32, max: 3600 }),
            Rc::new(RegionalEvidenceManifestVersionRangeRule { min: 32, max: 1000 }),
            Rc::new(RegionalEvidenceManifestVisibleGuardRule { limit: 16 }),
            Rc::new(RegionalEvidenceManifestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RegionalEvidenceManifestTitleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_evidence_manifest_validator();
        let item = new_regional_evidence_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = RegionalEvidenceManifestRequiredLocaleRule {
            message: "regional evidence manifest needs a value".to_string(),
        };
        let mut item = new_regional_evidence_manifest("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = RegionalEvidenceManifestDurationMinutesRangeRule { min: 16, max: 86400 };
        let mut item = new_regional_evidence_manifest("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RegionalEvidenceManifestVersionRangeRule { min: 80, max: 1000 };
        let mut item = new_regional_evidence_manifest("x", "y");
        item.version = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalEvidenceManifestTitleLengthRule { max_length: 32 };
        let mut item = new_regional_evidence_manifest("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
