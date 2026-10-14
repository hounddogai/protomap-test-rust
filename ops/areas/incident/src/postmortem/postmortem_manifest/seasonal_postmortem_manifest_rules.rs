use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal postmortem manifest must pass. Check returns its problems, or none.
pub trait SeasonalPostmortemManifestRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalPostmortemManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemManifestWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalPostmortemManifestRule for SeasonalPostmortemManifestWindowDaysRangeRule {
    fn check(&self, item: &SeasonalPostmortemManifest) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemManifestRequiredHostRule {
    pub message: String,
}

impl SeasonalPostmortemManifestRule for SeasonalPostmortemManifestRequiredHostRule {
    fn check(&self, item: &SeasonalPostmortemManifest) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemManifestHostLengthRule {
    pub max_length: i64,
}

impl SeasonalPostmortemManifestRule for SeasonalPostmortemManifestHostLengthRule {
    fn check(&self, item: &SeasonalPostmortemManifest) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemManifestAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl SeasonalPostmortemManifestRule for SeasonalPostmortemManifestAllowedAudienceRule {
    fn check(&self, item: &SeasonalPostmortemManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is above maximum", item.audience)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal postmortem manifest.
#[derive(Debug, Clone)]
pub struct SeasonalPostmortemManifestValidator {
    pub rules: Vec<Rc<dyn SeasonalPostmortemManifestRule>>,
}

impl SeasonalPostmortemManifestValidator {
    pub fn validate(&self, item: &SeasonalPostmortemManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalPostmortemManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalPostmortemManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_postmortem_manifest_validator() -> SeasonalPostmortemManifestValidator {
    SeasonalPostmortemManifestValidator {
        rules: vec![
            Rc::new(SeasonalPostmortemManifestWindowDaysRangeRule { min: 64, max: 1000 })
                as Rc<dyn SeasonalPostmortemManifestRule>,
            Rc::new(SeasonalPostmortemManifestRequiredHostRule {
                message: "seasonal postmortem manifest needs a value".to_string(),
            }),
            Rc::new(SeasonalPostmortemManifestHostLengthRule { max_length: 32 }),
            Rc::new(SeasonalPostmortemManifestAllowedAudienceRule {
                allowed: vec!["adaptive".to_string(), "capped".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_postmortem_manifest_validator();
        let item = new_seasonal_postmortem_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = SeasonalPostmortemManifestWindowDaysRangeRule { min: 16, max: 255 };
        let mut item = new_seasonal_postmortem_manifest("x", "y");
        item.window_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = SeasonalPostmortemManifestRequiredHostRule {
            message: "seasonal postmortem manifest needs this field".to_string(),
        };
        let mut item = new_seasonal_postmortem_manifest("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = SeasonalPostmortemManifestHostLengthRule { max_length: 86400 };
        let mut item = new_seasonal_postmortem_manifest("x", "y");
        item.host = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
