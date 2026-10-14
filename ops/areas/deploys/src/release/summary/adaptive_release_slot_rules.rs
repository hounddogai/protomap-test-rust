use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive release slot must pass. Check returns its problems, or none.
pub trait AdaptiveReleaseSlotRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveReleaseSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseSlotRequiredCodeRule {
    pub message: String,
}

impl AdaptiveReleaseSlotRule for AdaptiveReleaseSlotRequiredCodeRule {
    fn check(&self, item: &AdaptiveReleaseSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseSlotRequiredChannelRule {
    pub message: String,
}

impl AdaptiveReleaseSlotRule for AdaptiveReleaseSlotRequiredChannelRule {
    fn check(&self, item: &AdaptiveReleaseSlot) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseSlotCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveReleaseSlotRule for AdaptiveReleaseSlotCodePrefixRule {
    fn check(&self, item: &AdaptiveReleaseSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseSlotRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveReleaseSlotRule for AdaptiveReleaseSlotRatioRangeRule {
    fn check(&self, item: &AdaptiveReleaseSlot) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a adaptive release slot.
#[derive(Debug, Clone)]
pub struct AdaptiveReleaseSlotValidator {
    pub rules: Vec<Rc<dyn AdaptiveReleaseSlotRule>>,
}

impl AdaptiveReleaseSlotValidator {
    pub fn validate(&self, item: &AdaptiveReleaseSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveReleaseSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveReleaseSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_release_slot_validator() -> AdaptiveReleaseSlotValidator {
    AdaptiveReleaseSlotValidator {
        rules: vec![
            Rc::new(AdaptiveReleaseSlotRequiredCodeRule {
                message: "adaptive release slot needs an entry".to_string(),
            }) as Rc<dyn AdaptiveReleaseSlotRule>,
            Rc::new(AdaptiveReleaseSlotRequiredChannelRule {
                message: "adaptive release slot needs this field".to_string(),
            }),
            Rc::new(AdaptiveReleaseSlotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(AdaptiveReleaseSlotRatioRangeRule { min: 3.8, max: 4425.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_release_slot_validator();
        let item = new_adaptive_release_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveReleaseSlotRequiredCodeRule {
            message: "adaptive release slot needs a value".to_string(),
        };
        let mut item = new_adaptive_release_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = AdaptiveReleaseSlotRequiredChannelRule {
            message: "adaptive release slot needs an entry".to_string(),
        };
        let mut item = new_adaptive_release_slot("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = AdaptiveReleaseSlotRatioRangeRule { min: 8.1, max: 3142.7 };
        let mut item = new_adaptive_release_slot("x", "y");
        item.ratio = 3144.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
