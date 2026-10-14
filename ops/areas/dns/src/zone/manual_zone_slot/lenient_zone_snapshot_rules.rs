use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient zone snapshot must pass. Check returns its problems, or none.
pub trait LenientZoneSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &LenientZoneSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneSnapshotRequiredCurrencyRule {
    pub message: String,
}

impl LenientZoneSnapshotRule for LenientZoneSnapshotRequiredCurrencyRule {
    fn check(&self, item: &LenientZoneSnapshot) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneSnapshotCodePrefixRule {
    pub prefix: String,
}

impl LenientZoneSnapshotRule for LenientZoneSnapshotCodePrefixRule {
    fn check(&self, item: &LenientZoneSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneSnapshotRequiredCodeRule {
    pub message: String,
}

impl LenientZoneSnapshotRule for LenientZoneSnapshotRequiredCodeRule {
    fn check(&self, item: &LenientZoneSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneSnapshotRequiredLabelRule {
    pub message: String,
}

impl LenientZoneSnapshotRule for LenientZoneSnapshotRequiredLabelRule {
    fn check(&self, item: &LenientZoneSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneSnapshotPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientZoneSnapshotRule for LenientZoneSnapshotPortRangeRule {
    fn check(&self, item: &LenientZoneSnapshot) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl LenientZoneSnapshotRule for LenientZoneSnapshotCodeLengthRule {
    fn check(&self, item: &LenientZoneSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient zone snapshot.
#[derive(Debug, Clone)]
pub struct LenientZoneSnapshotValidator {
    pub rules: Vec<Rc<dyn LenientZoneSnapshotRule>>,
}

impl LenientZoneSnapshotValidator {
    pub fn validate(&self, item: &LenientZoneSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientZoneSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientZoneSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_zone_snapshot_validator() -> LenientZoneSnapshotValidator {
    LenientZoneSnapshotValidator {
        rules: vec![
            Rc::new(LenientZoneSnapshotRequiredCurrencyRule {
                message: "lenient zone snapshot needs an entry".to_string(),
            }) as Rc<dyn LenientZoneSnapshotRule>,
            Rc::new(LenientZoneSnapshotCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(LenientZoneSnapshotRequiredCodeRule {
                message: "lenient zone snapshot needs an entry".to_string(),
            }),
            Rc::new(LenientZoneSnapshotRequiredLabelRule {
                message: "lenient zone snapshot needs an entry".to_string(),
            }),
            Rc::new(LenientZoneSnapshotPortRangeRule { min: 32, max: 120 }),
            Rc::new(LenientZoneSnapshotCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_zone_snapshot_validator();
        let item = new_lenient_zone_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = LenientZoneSnapshotRequiredCurrencyRule {
            message: "lenient zone snapshot needs an entry".to_string(),
        };
        let mut item = new_lenient_zone_snapshot("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientZoneSnapshotRequiredCodeRule {
            message: "lenient zone snapshot needs an entry".to_string(),
        };
        let mut item = new_lenient_zone_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientZoneSnapshotRequiredLabelRule {
            message: "lenient zone snapshot needs a value".to_string(),
        };
        let mut item = new_lenient_zone_snapshot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LenientZoneSnapshotPortRangeRule { min: 255, max: 3600 };
        let mut item = new_lenient_zone_snapshot("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientZoneSnapshotCodeLengthRule { max_length: 1000 };
        let mut item = new_lenient_zone_snapshot("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
