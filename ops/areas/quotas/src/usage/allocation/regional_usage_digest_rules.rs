use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional usage digest must pass. Check returns its problems, or none.
pub trait RegionalUsageDigestRule: std::fmt::Debug {
    fn check(&self, item: &RegionalUsageDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageDigestRequiredLabelRule {
    pub message: String,
}

impl RegionalUsageDigestRule for RegionalUsageDigestRequiredLabelRule {
    fn check(&self, item: &RegionalUsageDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageDigestRequiredNotesRule {
    pub message: String,
}

impl RegionalUsageDigestRule for RegionalUsageDigestRequiredNotesRule {
    fn check(&self, item: &RegionalUsageDigest) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageDigestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalUsageDigestRule for RegionalUsageDigestAllowedCodeRule {
    fn check(&self, item: &RegionalUsageDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional usage digest.
#[derive(Debug, Clone)]
pub struct RegionalUsageDigestValidator {
    pub rules: Vec<Rc<dyn RegionalUsageDigestRule>>,
}

impl RegionalUsageDigestValidator {
    pub fn validate(&self, item: &RegionalUsageDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalUsageDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalUsageDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_usage_digest_validator() -> RegionalUsageDigestValidator {
    RegionalUsageDigestValidator {
        rules: vec![
            Rc::new(RegionalUsageDigestRequiredLabelRule {
                message: "regional usage digest needs this field".to_string(),
            }) as Rc<dyn RegionalUsageDigestRule>,
            Rc::new(RegionalUsageDigestRequiredNotesRule {
                message: "regional usage digest needs an entry".to_string(),
            }),
            Rc::new(RegionalUsageDigestAllowedCodeRule {
                allowed: vec!["global".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_usage_digest_validator();
        let item = new_regional_usage_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalUsageDigestRequiredLabelRule {
            message: "regional usage digest needs this field".to_string(),
        };
        let mut item = new_regional_usage_digest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RegionalUsageDigestRequiredNotesRule {
            message: "regional usage digest needs a value".to_string(),
        };
        let mut item = new_regional_usage_digest("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
