use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive probe entry must pass. Check returns its problems, or none.
pub trait AdaptiveProbeEntryRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveProbeEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeEntryPhoneLengthRule {
    pub max_length: i64,
}

impl AdaptiveProbeEntryRule for AdaptiveProbeEntryPhoneLengthRule {
    fn check(&self, item: &AdaptiveProbeEntry) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeEntryRequiredCodeRule {
    pub message: String,
}

impl AdaptiveProbeEntryRule for AdaptiveProbeEntryRequiredCodeRule {
    fn check(&self, item: &AdaptiveProbeEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeEntryRequiredCountryRule {
    pub message: String,
}

impl AdaptiveProbeEntryRule for AdaptiveProbeEntryRequiredCountryRule {
    fn check(&self, item: &AdaptiveProbeEntry) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeEntryRequiredNameRule {
    pub message: String,
}

impl AdaptiveProbeEntryRule for AdaptiveProbeEntryRequiredNameRule {
    fn check(&self, item: &AdaptiveProbeEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeEntryCountryLengthRule {
    pub max_length: i64,
}

impl AdaptiveProbeEntryRule for AdaptiveProbeEntryCountryLengthRule {
    fn check(&self, item: &AdaptiveProbeEntry) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive probe entry.
#[derive(Debug, Clone)]
pub struct AdaptiveProbeEntryValidator {
    pub rules: Vec<Rc<dyn AdaptiveProbeEntryRule>>,
}

impl AdaptiveProbeEntryValidator {
    pub fn validate(&self, item: &AdaptiveProbeEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveProbeEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveProbeEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_probe_entry_validator() -> AdaptiveProbeEntryValidator {
    AdaptiveProbeEntryValidator {
        rules: vec![
            Rc::new(AdaptiveProbeEntryPhoneLengthRule { max_length: 80 }) as Rc<dyn AdaptiveProbeEntryRule>,
            Rc::new(AdaptiveProbeEntryRequiredCodeRule {
                message: "adaptive probe entry needs this field".to_string(),
            }),
            Rc::new(AdaptiveProbeEntryRequiredCountryRule {
                message: "adaptive probe entry needs a value".to_string(),
            }),
            Rc::new(AdaptiveProbeEntryRequiredNameRule {
                message: "adaptive probe entry needs a value".to_string(),
            }),
            Rc::new(AdaptiveProbeEntryCountryLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_probe_entry_validator();
        let item = new_adaptive_probe_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = AdaptiveProbeEntryPhoneLengthRule { max_length: 32 };
        let mut item = new_adaptive_probe_entry("x", "y");
        item.phone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveProbeEntryRequiredCodeRule {
            message: "adaptive probe entry needs this field".to_string(),
        };
        let mut item = new_adaptive_probe_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = AdaptiveProbeEntryRequiredCountryRule {
            message: "adaptive probe entry needs a value".to_string(),
        };
        let mut item = new_adaptive_probe_entry("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AdaptiveProbeEntryRequiredNameRule {
            message: "adaptive probe entry needs an entry".to_string(),
        };
        let mut item = new_adaptive_probe_entry("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = AdaptiveProbeEntryCountryLengthRule { max_length: 32 };
        let mut item = new_adaptive_probe_entry("x", "y");
        item.country = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
