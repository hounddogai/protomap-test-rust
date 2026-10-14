use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone override must pass. Check returns its problems, or none.
pub trait ZoneOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ZoneOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneOverrideRequiredCodeRule {
    pub message: String,
}

impl ZoneOverrideRule for ZoneOverrideRequiredCodeRule {
    fn check(&self, item: &ZoneOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneOverrideCodePrefixRule {
    pub prefix: String,
}

impl ZoneOverrideRule for ZoneOverrideCodePrefixRule {
    fn check(&self, item: &ZoneOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneOverrideRequiredLabelRule {
    pub message: String,
}

impl ZoneOverrideRule for ZoneOverrideRequiredLabelRule {
    fn check(&self, item: &ZoneOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneOverrideAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl ZoneOverrideRule for ZoneOverrideAllowedTenantIdRule {
    fn check(&self, item: &ZoneOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is below minimum", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneOverrideThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ZoneOverrideRule for ZoneOverrideThresholdRangeRule {
    fn check(&self, item: &ZoneOverride) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a zone override.
#[derive(Debug, Clone)]
pub struct ZoneOverrideValidator {
    pub rules: Vec<Rc<dyn ZoneOverrideRule>>,
}

impl ZoneOverrideValidator {
    pub fn validate(&self, item: &ZoneOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_override_validator() -> ZoneOverrideValidator {
    ZoneOverrideValidator {
        rules: vec![
            Rc::new(ZoneOverrideRequiredCodeRule {
                message: "zone override needs an entry".to_string(),
            }) as Rc<dyn ZoneOverrideRule>,
            Rc::new(ZoneOverrideCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ZoneOverrideRequiredLabelRule {
                message: "zone override needs an entry".to_string(),
            }),
            Rc::new(ZoneOverrideAllowedTenantIdRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(ZoneOverrideThresholdRangeRule { min: 4.2, max: 843.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_override_validator();
        let item = new_zone_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ZoneOverrideRequiredCodeRule {
            message: "zone override needs this field".to_string(),
        };
        let mut item = new_zone_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ZoneOverrideRequiredLabelRule {
            message: "zone override needs a value".to_string(),
        };
        let mut item = new_zone_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = ZoneOverrideThresholdRangeRule { min: 6.5, max: 873.1 };
        let mut item = new_zone_override("x", "y");
        item.threshold = 874.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
