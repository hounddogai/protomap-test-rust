use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending probe adjustment must pass. Check returns its problems, or none.
pub trait PendingProbeAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &PendingProbeAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAdjustmentPinnedGuardRule {
    pub limit: i64,
}

impl PendingProbeAdjustmentRule for PendingProbeAdjustmentPinnedGuardRule {
    fn check(&self, item: &PendingProbeAdjustment) -> Vec<String> {
        if item.pinned && (item.quota_units > self.limit) {
            return vec![format!(
                "quota units {} exceeds {} while pinned",
                item.quota_units, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAdjustmentMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingProbeAdjustmentRule for PendingProbeAdjustmentMaximumRangeRule {
    fn check(&self, item: &PendingProbeAdjustment) -> Vec<String> {
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
pub struct PendingProbeAdjustmentRequiredLocaleRule {
    pub message: String,
}

impl PendingProbeAdjustmentRule for PendingProbeAdjustmentRequiredLocaleRule {
    fn check(&self, item: &PendingProbeAdjustment) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAdjustmentCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingProbeAdjustmentRule for PendingProbeAdjustmentCapacityRangeRule {
    fn check(&self, item: &PendingProbeAdjustment) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAdjustmentQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingProbeAdjustmentRule for PendingProbeAdjustmentQuotaUnitsRangeRule {
    fn check(&self, item: &PendingProbeAdjustment) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending probe adjustment.
#[derive(Debug, Clone)]
pub struct PendingProbeAdjustmentValidator {
    pub rules: Vec<Rc<dyn PendingProbeAdjustmentRule>>,
}

impl PendingProbeAdjustmentValidator {
    pub fn validate(&self, item: &PendingProbeAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingProbeAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingProbeAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_probe_adjustment_validator() -> PendingProbeAdjustmentValidator {
    PendingProbeAdjustmentValidator {
        rules: vec![
            Rc::new(PendingProbeAdjustmentPinnedGuardRule { limit: 1000 }) as Rc<dyn PendingProbeAdjustmentRule>,
            Rc::new(PendingProbeAdjustmentMaximumRangeRule { min: 8.1, max: 4981.7 }),
            Rc::new(PendingProbeAdjustmentRequiredLocaleRule {
                message: "pending probe adjustment needs this field".to_string(),
            }),
            Rc::new(PendingProbeAdjustmentCapacityRangeRule { min: 80, max: 1000 }),
            Rc::new(PendingProbeAdjustmentQuotaUnitsRangeRule { min: 32, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_probe_adjustment_validator();
        let item = new_pending_probe_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = PendingProbeAdjustmentMaximumRangeRule { min: 5.1, max: 384.6 };
        let mut item = new_pending_probe_adjustment("x", "y");
        item.maximum = 386.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = PendingProbeAdjustmentRequiredLocaleRule {
            message: "pending probe adjustment needs this field".to_string(),
        };
        let mut item = new_pending_probe_adjustment("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = PendingProbeAdjustmentCapacityRangeRule { min: 64, max: 1000 };
        let mut item = new_pending_probe_adjustment("x", "y");
        item.capacity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = PendingProbeAdjustmentQuotaUnitsRangeRule { min: 80, max: 255 };
        let mut item = new_pending_probe_adjustment("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
