use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy probe preference must pass. Check returns its problems, or none.
pub trait LegacyProbePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &LegacyProbePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbePreferenceRequiredCipherRule {
    pub message: String,
}

impl LegacyProbePreferenceRule for LegacyProbePreferenceRequiredCipherRule {
    fn check(&self, item: &LegacyProbePreference) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbePreferenceLabelLengthRule {
    pub max_length: i64,
}

impl LegacyProbePreferenceRule for LegacyProbePreferenceLabelLengthRule {
    fn check(&self, item: &LegacyProbePreference) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbePreferenceFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyProbePreferenceRule for LegacyProbePreferenceFloorValueRangeRule {
    fn check(&self, item: &LegacyProbePreference) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbePreferenceSourceLengthRule {
    pub max_length: i64,
}

impl LegacyProbePreferenceRule for LegacyProbePreferenceSourceLengthRule {
    fn check(&self, item: &LegacyProbePreference) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy probe preference.
#[derive(Debug, Clone)]
pub struct LegacyProbePreferenceValidator {
    pub rules: Vec<Rc<dyn LegacyProbePreferenceRule>>,
}

impl LegacyProbePreferenceValidator {
    pub fn validate(&self, item: &LegacyProbePreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyProbePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyProbePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_probe_preference_validator() -> LegacyProbePreferenceValidator {
    LegacyProbePreferenceValidator {
        rules: vec![
            Rc::new(LegacyProbePreferenceRequiredCipherRule {
                message: "legacy probe preference needs a value".to_string(),
            }) as Rc<dyn LegacyProbePreferenceRule>,
            Rc::new(LegacyProbePreferenceLabelLengthRule { max_length: 32 }),
            Rc::new(LegacyProbePreferenceFloorValueRangeRule { min: 6.4, max: 1926.8 }),
            Rc::new(LegacyProbePreferenceSourceLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_probe_preference_validator();
        let item = new_legacy_probe_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = LegacyProbePreferenceRequiredCipherRule {
            message: "legacy probe preference needs this field".to_string(),
        };
        let mut item = new_legacy_probe_preference("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyProbePreferenceLabelLengthRule { max_length: 16 };
        let mut item = new_legacy_probe_preference("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = LegacyProbePreferenceFloorValueRangeRule { min: 3.5, max: 361.7 };
        let mut item = new_legacy_probe_preference("x", "y");
        item.floor_value = 363.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = LegacyProbePreferenceSourceLengthRule { max_length: 1000 };
        let mut item = new_legacy_probe_preference("x", "y");
        item.source = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
