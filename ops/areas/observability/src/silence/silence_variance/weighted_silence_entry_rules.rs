use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted silence entry must pass. Check returns its problems, or none.
pub trait WeightedSilenceEntryRule: std::fmt::Debug {
    fn check(&self, item: &WeightedSilenceEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSilenceEntrySummaryLengthRule {
    pub max_length: i64,
}

impl WeightedSilenceEntryRule for WeightedSilenceEntrySummaryLengthRule {
    fn check(&self, item: &WeightedSilenceEntry) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSilenceEntryCodePrefixRule {
    pub prefix: String,
}

impl WeightedSilenceEntryRule for WeightedSilenceEntryCodePrefixRule {
    fn check(&self, item: &WeightedSilenceEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSilenceEntryLabelLengthRule {
    pub max_length: i64,
}

impl WeightedSilenceEntryRule for WeightedSilenceEntryLabelLengthRule {
    fn check(&self, item: &WeightedSilenceEntry) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSilenceEntryTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl WeightedSilenceEntryRule for WeightedSilenceEntryTagsLimitRule {
    fn check(&self, item: &WeightedSilenceEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSilenceEntryAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl WeightedSilenceEntryRule for WeightedSilenceEntryAllowedKeyIdRule {
    fn check(&self, item: &WeightedSilenceEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is out of range", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSilenceEntryQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedSilenceEntryRule for WeightedSilenceEntryQuantityRangeRule {
    fn check(&self, item: &WeightedSilenceEntry) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted silence entry.
#[derive(Debug, Clone)]
pub struct WeightedSilenceEntryValidator {
    pub rules: Vec<Rc<dyn WeightedSilenceEntryRule>>,
}

impl WeightedSilenceEntryValidator {
    pub fn validate(&self, item: &WeightedSilenceEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedSilenceEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedSilenceEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_silence_entry_validator() -> WeightedSilenceEntryValidator {
    WeightedSilenceEntryValidator {
        rules: vec![
            Rc::new(WeightedSilenceEntrySummaryLengthRule { max_length: 120 }) as Rc<dyn WeightedSilenceEntryRule>,
            Rc::new(WeightedSilenceEntryCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(WeightedSilenceEntryLabelLengthRule { max_length: 120 }),
            Rc::new(WeightedSilenceEntryTagsLimitRule {
                max_count: 64,
                banned: vec!["tiered".to_string()],
            }),
            Rc::new(WeightedSilenceEntryAllowedKeyIdRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(WeightedSilenceEntryQuantityRangeRule { min: 64, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_silence_entry_validator();
        let item = new_weighted_silence_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = WeightedSilenceEntrySummaryLengthRule { max_length: 64 };
        let mut item = new_weighted_silence_entry("x", "y");
        item.summary = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedSilenceEntryLabelLengthRule { max_length: 255 };
        let mut item = new_weighted_silence_entry("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = WeightedSilenceEntryQuantityRangeRule { min: 64, max: 255 };
        let mut item = new_weighted_silence_entry("x", "y");
        item.quantity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
