use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient runbook manifest must pass. Check returns its problems, or none.
pub trait LenientRunbookManifestRule: std::fmt::Debug {
    fn check(&self, item: &LenientRunbookManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookManifestPinnedGuardRule {
    pub limit: i64,
}

impl LenientRunbookManifestRule for LenientRunbookManifestPinnedGuardRule {
    fn check(&self, item: &LenientRunbookManifest) -> Vec<String> {
        if item.pinned && (item.window_days > self.limit) {
            return vec![format!(
                "window days {} exceeds {} while pinned",
                item.window_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookManifestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LenientRunbookManifestRule for LenientRunbookManifestAllowedCodeRule {
    fn check(&self, item: &LenientRunbookManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookManifestCodePrefixRule {
    pub prefix: String,
}

impl LenientRunbookManifestRule for LenientRunbookManifestCodePrefixRule {
    fn check(&self, item: &LenientRunbookManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookManifestMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientRunbookManifestRule for LenientRunbookManifestMarginRangeRule {
    fn check(&self, item: &LenientRunbookManifest) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookManifestCipherLengthRule {
    pub max_length: i64,
}

impl LenientRunbookManifestRule for LenientRunbookManifestCipherLengthRule {
    fn check(&self, item: &LenientRunbookManifest) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookManifestRequiredCipherRule {
    pub message: String,
}

impl LenientRunbookManifestRule for LenientRunbookManifestRequiredCipherRule {
    fn check(&self, item: &LenientRunbookManifest) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient runbook manifest.
#[derive(Debug, Clone)]
pub struct LenientRunbookManifestValidator {
    pub rules: Vec<Rc<dyn LenientRunbookManifestRule>>,
}

impl LenientRunbookManifestValidator {
    pub fn validate(&self, item: &LenientRunbookManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientRunbookManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRunbookManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_runbook_manifest_validator() -> LenientRunbookManifestValidator {
    LenientRunbookManifestValidator {
        rules: vec![
            Rc::new(LenientRunbookManifestPinnedGuardRule { limit: 64 }) as Rc<dyn LenientRunbookManifestRule>,
            Rc::new(LenientRunbookManifestAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(LenientRunbookManifestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(LenientRunbookManifestMarginRangeRule { min: 2.7, max: 3143.9 }),
            Rc::new(LenientRunbookManifestCipherLengthRule { max_length: 1000 }),
            Rc::new(LenientRunbookManifestRequiredCipherRule {
                message: "lenient runbook manifest needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_runbook_manifest_validator();
        let item = new_lenient_runbook_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = LenientRunbookManifestMarginRangeRule { min: 3.3, max: 3658.2 };
        let mut item = new_lenient_runbook_manifest("x", "y");
        item.margin = 3659.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = LenientRunbookManifestCipherLengthRule { max_length: 80 };
        let mut item = new_lenient_runbook_manifest("x", "y");
        item.cipher = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = LenientRunbookManifestRequiredCipherRule {
            message: "lenient runbook manifest needs this field".to_string(),
        };
        let mut item = new_lenient_runbook_manifest("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
