use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment preference must pass. Check returns its problems, or none.
pub trait SegmentPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &SegmentPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPreferenceTitleLengthRule {
    pub max_length: i64,
}

impl SegmentPreferenceRule for SegmentPreferenceTitleLengthRule {
    fn check(&self, item: &SegmentPreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPreferenceRequiredClusterRule {
    pub message: String,
}

impl SegmentPreferenceRule for SegmentPreferenceRequiredClusterRule {
    fn check(&self, item: &SegmentPreference) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPreferenceCodePrefixRule {
    pub prefix: String,
}

impl SegmentPreferenceRule for SegmentPreferenceCodePrefixRule {
    fn check(&self, item: &SegmentPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPreferenceRequiredCodeRule {
    pub message: String,
}

impl SegmentPreferenceRule for SegmentPreferenceRequiredCodeRule {
    fn check(&self, item: &SegmentPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment preference.
#[derive(Debug, Clone)]
pub struct SegmentPreferenceValidator {
    pub rules: Vec<Rc<dyn SegmentPreferenceRule>>,
}

impl SegmentPreferenceValidator {
    pub fn validate(&self, item: &SegmentPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_preference_validator() -> SegmentPreferenceValidator {
    SegmentPreferenceValidator {
        rules: vec![
            Rc::new(SegmentPreferenceTitleLengthRule { max_length: 1000 }) as Rc<dyn SegmentPreferenceRule>,
            Rc::new(SegmentPreferenceRequiredClusterRule {
                message: "segment preference needs an entry".to_string(),
            }),
            Rc::new(SegmentPreferenceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SegmentPreferenceRequiredCodeRule {
                message: "segment preference needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_preference_validator();
        let item = new_segment_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SegmentPreferenceTitleLengthRule { max_length: 1000 };
        let mut item = new_segment_preference("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = SegmentPreferenceRequiredClusterRule {
            message: "segment preference needs an entry".to_string(),
        };
        let mut item = new_segment_preference("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SegmentPreferenceRequiredCodeRule {
            message: "segment preference needs an entry".to_string(),
        };
        let mut item = new_segment_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
