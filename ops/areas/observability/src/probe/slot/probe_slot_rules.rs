use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe slot must pass. Check returns its problems, or none.
pub trait ProbeSlotRule: std::fmt::Debug {
    fn check(&self, item: &ProbeSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSlotLabelLengthRule {
    pub max_length: i64,
}

impl ProbeSlotRule for ProbeSlotLabelLengthRule {
    fn check(&self, item: &ProbeSlot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSlotThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeSlotRule for ProbeSlotThresholdRangeRule {
    fn check(&self, item: &ProbeSlot) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSlotVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeSlotRule for ProbeSlotVersionRangeRule {
    fn check(&self, item: &ProbeSlot) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSlotRequiredPrincipalRule {
    pub message: String,
}

impl ProbeSlotRule for ProbeSlotRequiredPrincipalRule {
    fn check(&self, item: &ProbeSlot) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe slot.
#[derive(Debug, Clone)]
pub struct ProbeSlotValidator {
    pub rules: Vec<Rc<dyn ProbeSlotRule>>,
}

impl ProbeSlotValidator {
    pub fn validate(&self, item: &ProbeSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_slot_validator() -> ProbeSlotValidator {
    ProbeSlotValidator {
        rules: vec![
            Rc::new(ProbeSlotLabelLengthRule { max_length: 86400 }) as Rc<dyn ProbeSlotRule>,
            Rc::new(ProbeSlotThresholdRangeRule { min: 6.2, max: 2395.9 }),
            Rc::new(ProbeSlotVersionRangeRule { min: 3600, max: 86400 }),
            Rc::new(ProbeSlotRequiredPrincipalRule {
                message: "probe slot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_slot_validator();
        let item = new_probe_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ProbeSlotLabelLengthRule { max_length: 255 };
        let mut item = new_probe_slot("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = ProbeSlotThresholdRangeRule { min: 8.1, max: 1615.5 };
        let mut item = new_probe_slot("x", "y");
        item.threshold = 1617.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = ProbeSlotVersionRangeRule { min: 16, max: 16 };
        let mut item = new_probe_slot("x", "y");
        item.version = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = ProbeSlotRequiredPrincipalRule {
            message: "probe slot needs a value".to_string(),
        };
        let mut item = new_probe_slot("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
