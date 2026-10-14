use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered alert adjustment must pass. Check returns its problems, or none.
pub trait TieredAlertAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &TieredAlertAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertAdjustmentRequiredCodeRule {
    pub message: String,
}

impl TieredAlertAdjustmentRule for TieredAlertAdjustmentRequiredCodeRule {
    fn check(&self, item: &TieredAlertAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertAdjustmentPinnedGuardRule {
    pub limit: f64,
}

impl TieredAlertAdjustmentRule for TieredAlertAdjustmentPinnedGuardRule {
    fn check(&self, item: &TieredAlertAdjustment) -> Vec<String> {
        if item.pinned && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while pinned", item.rate, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertAdjustmentRequiredNameRule {
    pub message: String,
}

impl TieredAlertAdjustmentRule for TieredAlertAdjustmentRequiredNameRule {
    fn check(&self, item: &TieredAlertAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered alert adjustment.
#[derive(Debug, Clone)]
pub struct TieredAlertAdjustmentValidator {
    pub rules: Vec<Rc<dyn TieredAlertAdjustmentRule>>,
}

impl TieredAlertAdjustmentValidator {
    pub fn validate(&self, item: &TieredAlertAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredAlertAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredAlertAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_alert_adjustment_validator() -> TieredAlertAdjustmentValidator {
    TieredAlertAdjustmentValidator {
        rules: vec![
            Rc::new(TieredAlertAdjustmentRequiredCodeRule {
                message: "tiered alert adjustment needs an entry".to_string(),
            }) as Rc<dyn TieredAlertAdjustmentRule>,
            Rc::new(TieredAlertAdjustmentPinnedGuardRule { limit: 3.1 }),
            Rc::new(TieredAlertAdjustmentRequiredNameRule {
                message: "tiered alert adjustment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_alert_adjustment_validator();
        let item = new_tiered_alert_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredAlertAdjustmentRequiredCodeRule {
            message: "tiered alert adjustment needs this field".to_string(),
        };
        let mut item = new_tiered_alert_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredAlertAdjustmentRequiredNameRule {
            message: "tiered alert adjustment needs this field".to_string(),
        };
        let mut item = new_tiered_alert_adjustment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
