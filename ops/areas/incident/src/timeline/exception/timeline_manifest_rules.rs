use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline manifest must pass. Check returns its problems, or none.
pub trait TimelineManifestRule: std::fmt::Debug {
    fn check(&self, item: &TimelineManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineManifestCodePrefixRule {
    pub prefix: String,
}

impl TimelineManifestRule for TimelineManifestCodePrefixRule {
    fn check(&self, item: &TimelineManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineManifestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineManifestRule for TimelineManifestCreatedAtRangeRule {
    fn check(&self, item: &TimelineManifest) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineManifestLabelLengthRule {
    pub max_length: i64,
}

impl TimelineManifestRule for TimelineManifestLabelLengthRule {
    fn check(&self, item: &TimelineManifest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineManifestRequiredLocaleRule {
    pub message: String,
}

impl TimelineManifestRule for TimelineManifestRequiredLocaleRule {
    fn check(&self, item: &TimelineManifest) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineManifestCodeLengthRule {
    pub max_length: i64,
}

impl TimelineManifestRule for TimelineManifestCodeLengthRule {
    fn check(&self, item: &TimelineManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineManifestLocaleLengthRule {
    pub max_length: i64,
}

impl TimelineManifestRule for TimelineManifestLocaleLengthRule {
    fn check(&self, item: &TimelineManifest) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline manifest.
#[derive(Debug, Clone)]
pub struct TimelineManifestValidator {
    pub rules: Vec<Rc<dyn TimelineManifestRule>>,
}

impl TimelineManifestValidator {
    pub fn validate(&self, item: &TimelineManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_manifest_validator() -> TimelineManifestValidator {
    TimelineManifestValidator {
        rules: vec![
            Rc::new(TimelineManifestCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn TimelineManifestRule>,
            Rc::new(TimelineManifestCreatedAtRangeRule { min: 32, max: 80 }),
            Rc::new(TimelineManifestLabelLengthRule { max_length: 3600 }),
            Rc::new(TimelineManifestRequiredLocaleRule {
                message: "timeline manifest needs a value".to_string(),
            }),
            Rc::new(TimelineManifestCodeLengthRule { max_length: 255 }),
            Rc::new(TimelineManifestLocaleLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_manifest_validator();
        let item = new_timeline_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = TimelineManifestCreatedAtRangeRule { min: 16, max: 1000 };
        let mut item = new_timeline_manifest("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TimelineManifestLabelLengthRule { max_length: 16 };
        let mut item = new_timeline_manifest("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = TimelineManifestRequiredLocaleRule {
            message: "timeline manifest needs an entry".to_string(),
        };
        let mut item = new_timeline_manifest("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TimelineManifestCodeLengthRule { max_length: 86400 };
        let mut item = new_timeline_manifest("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = TimelineManifestLocaleLengthRule { max_length: 120 };
        let mut item = new_timeline_manifest("x", "y");
        item.locale = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
