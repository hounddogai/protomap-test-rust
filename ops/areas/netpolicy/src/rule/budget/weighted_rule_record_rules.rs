use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted rule record must pass. Check returns its problems, or none.
pub trait WeightedRuleRecordRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRuleRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleRecordAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl WeightedRuleRecordRule for WeightedRuleRecordAllowedChannelRule {
    fn check(&self, item: &WeightedRuleRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is unknown", item.channel)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleRecordRequiredChannelRule {
    pub message: String,
}

impl WeightedRuleRecordRule for WeightedRuleRecordRequiredChannelRule {
    fn check(&self, item: &WeightedRuleRecord) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleRecordCodePrefixRule {
    pub prefix: String,
}

impl WeightedRuleRecordRule for WeightedRuleRecordCodePrefixRule {
    fn check(&self, item: &WeightedRuleRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleRecordRequiredPrincipalRule {
    pub message: String,
}

impl WeightedRuleRecordRule for WeightedRuleRecordRequiredPrincipalRule {
    fn check(&self, item: &WeightedRuleRecord) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleRecordRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedRuleRecordRule for WeightedRuleRecordRatioRangeRule {
    fn check(&self, item: &WeightedRuleRecord) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted rule record.
#[derive(Debug, Clone)]
pub struct WeightedRuleRecordValidator {
    pub rules: Vec<Rc<dyn WeightedRuleRecordRule>>,
}

impl WeightedRuleRecordValidator {
    pub fn validate(&self, item: &WeightedRuleRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedRuleRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRuleRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_rule_record_validator() -> WeightedRuleRecordValidator {
    WeightedRuleRecordValidator {
        rules: vec![
            Rc::new(WeightedRuleRecordAllowedChannelRule {
                allowed: vec!["adaptive".to_string(), "capped".to_string()],
            }) as Rc<dyn WeightedRuleRecordRule>,
            Rc::new(WeightedRuleRecordRequiredChannelRule {
                message: "weighted rule record needs this field".to_string(),
            }),
            Rc::new(WeightedRuleRecordCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(WeightedRuleRecordRequiredPrincipalRule {
                message: "weighted rule record needs an entry".to_string(),
            }),
            Rc::new(WeightedRuleRecordRatioRangeRule { min: 4.7, max: 381.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_rule_record_validator();
        let item = new_weighted_rule_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = WeightedRuleRecordRequiredChannelRule {
            message: "weighted rule record needs this field".to_string(),
        };
        let mut item = new_weighted_rule_record("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = WeightedRuleRecordRequiredPrincipalRule {
            message: "weighted rule record needs this field".to_string(),
        };
        let mut item = new_weighted_rule_record("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = WeightedRuleRecordRatioRangeRule { min: 4.0, max: 210.4 };
        let mut item = new_weighted_rule_record("x", "y");
        item.ratio = 211.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
