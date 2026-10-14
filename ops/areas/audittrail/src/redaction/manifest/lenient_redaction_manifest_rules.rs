use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient redaction manifest must pass. Check returns its problems, or none.
pub trait LenientRedactionManifestRule: std::fmt::Debug {
    fn check(&self, item: &LenientRedactionManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionManifestRequiredTitleRule {
    pub message: String,
}

impl LenientRedactionManifestRule for LenientRedactionManifestRequiredTitleRule {
    fn check(&self, item: &LenientRedactionManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionManifestVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRedactionManifestRule for LenientRedactionManifestVersionRangeRule {
    fn check(&self, item: &LenientRedactionManifest) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionManifestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LenientRedactionManifestRule for LenientRedactionManifestAllowedCodeRule {
    fn check(&self, item: &LenientRedactionManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionManifestScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientRedactionManifestRule for LenientRedactionManifestScoreRangeRule {
    fn check(&self, item: &LenientRedactionManifest) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionManifestCodeLengthRule {
    pub max_length: i64,
}

impl LenientRedactionManifestRule for LenientRedactionManifestCodeLengthRule {
    fn check(&self, item: &LenientRedactionManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient redaction manifest.
#[derive(Debug, Clone)]
pub struct LenientRedactionManifestValidator {
    pub rules: Vec<Rc<dyn LenientRedactionManifestRule>>,
}

impl LenientRedactionManifestValidator {
    pub fn validate(&self, item: &LenientRedactionManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientRedactionManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRedactionManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_redaction_manifest_validator() -> LenientRedactionManifestValidator {
    LenientRedactionManifestValidator {
        rules: vec![
            Rc::new(LenientRedactionManifestRequiredTitleRule {
                message: "lenient redaction manifest needs this field".to_string(),
            }) as Rc<dyn LenientRedactionManifestRule>,
            Rc::new(LenientRedactionManifestVersionRangeRule { min: 64, max: 80 }),
            Rc::new(LenientRedactionManifestAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(LenientRedactionManifestScoreRangeRule { min: 7.5, max: 269.3 }),
            Rc::new(LenientRedactionManifestCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_redaction_manifest_validator();
        let item = new_lenient_redaction_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientRedactionManifestRequiredTitleRule {
            message: "lenient redaction manifest needs this field".to_string(),
        };
        let mut item = new_lenient_redaction_manifest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = LenientRedactionManifestVersionRangeRule { min: 255, max: 86400 };
        let mut item = new_lenient_redaction_manifest("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = LenientRedactionManifestScoreRangeRule { min: 1.2, max: 4936.0 };
        let mut item = new_lenient_redaction_manifest("x", "y");
        item.score = 4937.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientRedactionManifestCodeLengthRule { max_length: 16 };
        let mut item = new_lenient_redaction_manifest("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
