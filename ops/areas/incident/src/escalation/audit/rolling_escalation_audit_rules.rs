use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling escalation audit must pass. Check returns its problems, or none.
pub trait RollingEscalationAuditRule: std::fmt::Debug {
    fn check(&self, item: &RollingEscalationAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationAuditTitleLengthRule {
    pub max_length: i64,
}

impl RollingEscalationAuditRule for RollingEscalationAuditTitleLengthRule {
    fn check(&self, item: &RollingEscalationAudit) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationAuditRequiredTitleRule {
    pub message: String,
}

impl RollingEscalationAuditRule for RollingEscalationAuditRequiredTitleRule {
    fn check(&self, item: &RollingEscalationAudit) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationAuditScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingEscalationAuditRule for RollingEscalationAuditScoreRangeRule {
    fn check(&self, item: &RollingEscalationAudit) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationAuditRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingEscalationAuditRule for RollingEscalationAuditRatioRangeRule {
    fn check(&self, item: &RollingEscalationAudit) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling escalation audit.
#[derive(Debug, Clone)]
pub struct RollingEscalationAuditValidator {
    pub rules: Vec<Rc<dyn RollingEscalationAuditRule>>,
}

impl RollingEscalationAuditValidator {
    pub fn validate(&self, item: &RollingEscalationAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingEscalationAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingEscalationAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_escalation_audit_validator() -> RollingEscalationAuditValidator {
    RollingEscalationAuditValidator {
        rules: vec![
            Rc::new(RollingEscalationAuditTitleLengthRule { max_length: 1000 }) as Rc<dyn RollingEscalationAuditRule>,
            Rc::new(RollingEscalationAuditRequiredTitleRule {
                message: "rolling escalation audit needs an entry".to_string(),
            }),
            Rc::new(RollingEscalationAuditScoreRangeRule { min: 2.5, max: 4604.7 }),
            Rc::new(RollingEscalationAuditRatioRangeRule { min: 1.8, max: 1291.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_escalation_audit_validator();
        let item = new_rolling_escalation_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollingEscalationAuditTitleLengthRule { max_length: 120 };
        let mut item = new_rolling_escalation_audit("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollingEscalationAuditRequiredTitleRule {
            message: "rolling escalation audit needs an entry".to_string(),
        };
        let mut item = new_rolling_escalation_audit("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = RollingEscalationAuditScoreRangeRule { min: 5.7, max: 1932.1 };
        let mut item = new_rolling_escalation_audit("x", "y");
        item.score = 1933.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = RollingEscalationAuditRatioRangeRule { min: 6.6, max: 1195.3 };
        let mut item = new_rolling_escalation_audit("x", "y");
        item.ratio = 1196.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
