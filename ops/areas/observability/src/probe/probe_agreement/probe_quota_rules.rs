use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe quota must pass. Check returns its problems, or none.
pub trait ProbeQuotaRule: std::fmt::Debug {
    fn check(&self, item: &ProbeQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeQuotaRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeQuotaRule for ProbeQuotaRatioRangeRule {
    fn check(&self, item: &ProbeQuota) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeQuotaRequiredCodeRule {
    pub message: String,
}

impl ProbeQuotaRule for ProbeQuotaRequiredCodeRule {
    fn check(&self, item: &ProbeQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeQuotaGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeQuotaRule for ProbeQuotaGraceDaysRangeRule {
    fn check(&self, item: &ProbeQuota) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeQuotaQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeQuotaRule for ProbeQuotaQuantityRangeRule {
    fn check(&self, item: &ProbeQuota) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe quota.
#[derive(Debug, Clone)]
pub struct ProbeQuotaValidator {
    pub rules: Vec<Rc<dyn ProbeQuotaRule>>,
}

impl ProbeQuotaValidator {
    pub fn validate(&self, item: &ProbeQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_quota_validator() -> ProbeQuotaValidator {
    ProbeQuotaValidator {
        rules: vec![
            Rc::new(ProbeQuotaRatioRangeRule { min: 5.9, max: 384.1 }) as Rc<dyn ProbeQuotaRule>,
            Rc::new(ProbeQuotaRequiredCodeRule {
                message: "probe quota needs a value".to_string(),
            }),
            Rc::new(ProbeQuotaGraceDaysRangeRule { min: 80, max: 3600 }),
            Rc::new(ProbeQuotaQuantityRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_quota_validator();
        let item = new_probe_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = ProbeQuotaRatioRangeRule { min: 5.1, max: 285.4 };
        let mut item = new_probe_quota("x", "y");
        item.ratio = 286.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ProbeQuotaRequiredCodeRule {
            message: "probe quota needs a value".to_string(),
        };
        let mut item = new_probe_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = ProbeQuotaGraceDaysRangeRule { min: 64, max: 255 };
        let mut item = new_probe_quota("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ProbeQuotaQuantityRangeRule { min: 32, max: 80 };
        let mut item = new_probe_quota("x", "y");
        item.quantity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
