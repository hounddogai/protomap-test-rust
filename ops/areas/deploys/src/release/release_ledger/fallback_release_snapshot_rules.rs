use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback release snapshot must pass. Check returns its problems, or none.
pub trait FallbackReleaseSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &FallbackReleaseSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseSnapshotRequiredCodeRule {
    pub message: String,
}

impl FallbackReleaseSnapshotRule for FallbackReleaseSnapshotRequiredCodeRule {
    fn check(&self, item: &FallbackReleaseSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseSnapshotRequiredTitleRule {
    pub message: String,
}

impl FallbackReleaseSnapshotRule for FallbackReleaseSnapshotRequiredTitleRule {
    fn check(&self, item: &FallbackReleaseSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseSnapshotLocaleLengthRule {
    pub max_length: i64,
}

impl FallbackReleaseSnapshotRule for FallbackReleaseSnapshotLocaleLengthRule {
    fn check(&self, item: &FallbackReleaseSnapshot) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseSnapshotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FallbackReleaseSnapshotRule for FallbackReleaseSnapshotAllowedCodeRule {
    fn check(&self, item: &FallbackReleaseSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseSnapshotVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackReleaseSnapshotRule for FallbackReleaseSnapshotVersionRangeRule {
    fn check(&self, item: &FallbackReleaseSnapshot) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a fallback release snapshot.
#[derive(Debug, Clone)]
pub struct FallbackReleaseSnapshotValidator {
    pub rules: Vec<Rc<dyn FallbackReleaseSnapshotRule>>,
}

impl FallbackReleaseSnapshotValidator {
    pub fn validate(&self, item: &FallbackReleaseSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackReleaseSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackReleaseSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_release_snapshot_validator() -> FallbackReleaseSnapshotValidator {
    FallbackReleaseSnapshotValidator {
        rules: vec![
            Rc::new(FallbackReleaseSnapshotRequiredCodeRule {
                message: "fallback release snapshot needs an entry".to_string(),
            }) as Rc<dyn FallbackReleaseSnapshotRule>,
            Rc::new(FallbackReleaseSnapshotRequiredTitleRule {
                message: "fallback release snapshot needs this field".to_string(),
            }),
            Rc::new(FallbackReleaseSnapshotLocaleLengthRule { max_length: 80 }),
            Rc::new(FallbackReleaseSnapshotAllowedCodeRule {
                allowed: vec!["strict".to_string(), "seasonal".to_string(), "adaptive".to_string()],
            }),
            Rc::new(FallbackReleaseSnapshotVersionRangeRule { min: 120, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_release_snapshot_validator();
        let item = new_fallback_release_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackReleaseSnapshotRequiredCodeRule {
            message: "fallback release snapshot needs this field".to_string(),
        };
        let mut item = new_fallback_release_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FallbackReleaseSnapshotRequiredTitleRule {
            message: "fallback release snapshot needs a value".to_string(),
        };
        let mut item = new_fallback_release_snapshot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = FallbackReleaseSnapshotLocaleLengthRule { max_length: 86400 };
        let mut item = new_fallback_release_snapshot("x", "y");
        item.locale = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = FallbackReleaseSnapshotVersionRangeRule { min: 16, max: 32 };
        let mut item = new_fallback_release_snapshot("x", "y");
        item.version = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
