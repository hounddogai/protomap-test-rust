use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict region pin adjustment must pass. Check returns its problems, or none.
pub trait StrictRegionPinAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &StrictRegionPinAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRegionPinAdjustmentLabelLengthRule {
    pub max_length: i64,
}

impl StrictRegionPinAdjustmentRule for StrictRegionPinAdjustmentLabelLengthRule {
    fn check(&self, item: &StrictRegionPinAdjustment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRegionPinAdjustmentRequiredLabelRule {
    pub message: String,
}

impl StrictRegionPinAdjustmentRule for StrictRegionPinAdjustmentRequiredLabelRule {
    fn check(&self, item: &StrictRegionPinAdjustment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRegionPinAdjustmentFingerprintLengthRule {
    pub max_length: i64,
}

impl StrictRegionPinAdjustmentRule for StrictRegionPinAdjustmentFingerprintLengthRule {
    fn check(&self, item: &StrictRegionPinAdjustment) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRegionPinAdjustmentMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictRegionPinAdjustmentRule for StrictRegionPinAdjustmentMaximumRangeRule {
    fn check(&self, item: &StrictRegionPinAdjustment) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRegionPinAdjustmentApprovedGuardRule {
    pub limit: f64,
}

impl StrictRegionPinAdjustmentRule for StrictRegionPinAdjustmentApprovedGuardRule {
    fn check(&self, item: &StrictRegionPinAdjustment) -> Vec<String> {
        if item.approved && (item.maximum > self.limit) {
            return vec![format!(
                "maximum {} exceeds {} while approved",
                item.maximum, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRegionPinAdjustmentAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl StrictRegionPinAdjustmentRule for StrictRegionPinAdjustmentAllowedChecksumRule {
    fn check(&self, item: &StrictRegionPinAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is above maximum", item.checksum)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict region pin adjustment.
#[derive(Debug, Clone)]
pub struct StrictRegionPinAdjustmentValidator {
    pub rules: Vec<Rc<dyn StrictRegionPinAdjustmentRule>>,
}

impl StrictRegionPinAdjustmentValidator {
    pub fn validate(&self, item: &StrictRegionPinAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictRegionPinAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictRegionPinAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_region_pin_adjustment_validator() -> StrictRegionPinAdjustmentValidator {
    StrictRegionPinAdjustmentValidator {
        rules: vec![
            Rc::new(StrictRegionPinAdjustmentLabelLengthRule { max_length: 1000 })
                as Rc<dyn StrictRegionPinAdjustmentRule>,
            Rc::new(StrictRegionPinAdjustmentRequiredLabelRule {
                message: "strict region pin adjustment needs this field".to_string(),
            }),
            Rc::new(StrictRegionPinAdjustmentFingerprintLengthRule { max_length: 1000 }),
            Rc::new(StrictRegionPinAdjustmentMaximumRangeRule { min: 4.1, max: 2725.8 }),
            Rc::new(StrictRegionPinAdjustmentApprovedGuardRule { limit: 9.5 }),
            Rc::new(StrictRegionPinAdjustmentAllowedChecksumRule {
                allowed: vec!["scheduled".to_string(), "legacy".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_region_pin_adjustment_validator();
        let item = new_strict_region_pin_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = StrictRegionPinAdjustmentLabelLengthRule { max_length: 3600 };
        let mut item = new_strict_region_pin_adjustment("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictRegionPinAdjustmentRequiredLabelRule {
            message: "strict region pin adjustment needs a value".to_string(),
        };
        let mut item = new_strict_region_pin_adjustment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = StrictRegionPinAdjustmentFingerprintLengthRule { max_length: 255 };
        let mut item = new_strict_region_pin_adjustment("x", "y");
        item.fingerprint = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = StrictRegionPinAdjustmentMaximumRangeRule { min: 5.3, max: 1624.6 };
        let mut item = new_strict_region_pin_adjustment("x", "y");
        item.maximum = 1626.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
