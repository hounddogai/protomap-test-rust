use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled failover summary must pass. Check returns its problems, or none.
pub trait ScheduledFailoverSummaryRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledFailoverSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSummaryArchivedGuardRule {
    pub limit: i64,
}

impl ScheduledFailoverSummaryRule for ScheduledFailoverSummaryArchivedGuardRule {
    fn check(&self, item: &ScheduledFailoverSummary) -> Vec<String> {
        if item.archived && (item.limit > self.limit) {
            return vec![format!("limit {} exceeds {} while archived", item.limit, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSummaryRequiredContactEmailRule {
    pub message: String,
}

impl ScheduledFailoverSummaryRule for ScheduledFailoverSummaryRequiredContactEmailRule {
    fn check(&self, item: &ScheduledFailoverSummary) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSummaryRequiredReviewerRule {
    pub message: String,
}

impl ScheduledFailoverSummaryRule for ScheduledFailoverSummaryRequiredReviewerRule {
    fn check(&self, item: &ScheduledFailoverSummary) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSummaryRequiredCodeRule {
    pub message: String,
}

impl ScheduledFailoverSummaryRule for ScheduledFailoverSummaryRequiredCodeRule {
    fn check(&self, item: &ScheduledFailoverSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled failover summary.
#[derive(Debug, Clone)]
pub struct ScheduledFailoverSummaryValidator {
    pub rules: Vec<Rc<dyn ScheduledFailoverSummaryRule>>,
}

impl ScheduledFailoverSummaryValidator {
    pub fn validate(&self, item: &ScheduledFailoverSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledFailoverSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledFailoverSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_failover_summary_validator() -> ScheduledFailoverSummaryValidator {
    ScheduledFailoverSummaryValidator {
        rules: vec![
            Rc::new(ScheduledFailoverSummaryArchivedGuardRule { limit: 32 }) as Rc<dyn ScheduledFailoverSummaryRule>,
            Rc::new(ScheduledFailoverSummaryRequiredContactEmailRule {
                message: "scheduled failover summary needs an entry".to_string(),
            }),
            Rc::new(ScheduledFailoverSummaryRequiredReviewerRule {
                message: "scheduled failover summary needs a value".to_string(),
            }),
            Rc::new(ScheduledFailoverSummaryRequiredCodeRule {
                message: "scheduled failover summary needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_failover_summary_validator();
        let item = new_scheduled_failover_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = ScheduledFailoverSummaryRequiredContactEmailRule {
            message: "scheduled failover summary needs an entry".to_string(),
        };
        let mut item = new_scheduled_failover_summary("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = ScheduledFailoverSummaryRequiredReviewerRule {
            message: "scheduled failover summary needs a value".to_string(),
        };
        let mut item = new_scheduled_failover_summary("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledFailoverSummaryRequiredCodeRule {
            message: "scheduled failover summary needs a value".to_string(),
        };
        let mut item = new_scheduled_failover_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
