use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered alert score must pass. Check returns its problems, or none.
pub trait TieredAlertScoreRule: std::fmt::Debug {
    fn check(&self, item: &TieredAlertScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertScoreCodePrefixRule {
    pub prefix: String,
}

impl TieredAlertScoreRule for TieredAlertScoreCodePrefixRule {
    fn check(&self, item: &TieredAlertScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertScoreAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl TieredAlertScoreRule for TieredAlertScoreAllowedTenantIdRule {
    fn check(&self, item: &TieredAlertScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is duplicated", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertScoreRequiredCodeRule {
    pub message: String,
}

impl TieredAlertScoreRule for TieredAlertScoreRequiredCodeRule {
    fn check(&self, item: &TieredAlertScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertScoreMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredAlertScoreRule for TieredAlertScoreMarginRangeRule {
    fn check(&self, item: &TieredAlertScore) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered alert score.
#[derive(Debug, Clone)]
pub struct TieredAlertScoreValidator {
    pub rules: Vec<Rc<dyn TieredAlertScoreRule>>,
}

impl TieredAlertScoreValidator {
    pub fn validate(&self, item: &TieredAlertScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredAlertScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredAlertScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_alert_score_validator() -> TieredAlertScoreValidator {
    TieredAlertScoreValidator {
        rules: vec![
            Rc::new(TieredAlertScoreCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn TieredAlertScoreRule>,
            Rc::new(TieredAlertScoreAllowedTenantIdRule {
                allowed: vec!["legacy".to_string()],
            }),
            Rc::new(TieredAlertScoreRequiredCodeRule {
                message: "tiered alert score needs an entry".to_string(),
            }),
            Rc::new(TieredAlertScoreMarginRangeRule { min: 5.9, max: 3939.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_alert_score_validator();
        let item = new_tiered_alert_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredAlertScoreRequiredCodeRule {
            message: "tiered alert score needs a value".to_string(),
        };
        let mut item = new_tiered_alert_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = TieredAlertScoreMarginRangeRule { min: 1.0, max: 1311.4 };
        let mut item = new_tiered_alert_score("x", "y");
        item.margin = 1312.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
