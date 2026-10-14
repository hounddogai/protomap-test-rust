use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence notice must pass. Check returns its problems, or none.
pub trait EvidenceNoticeRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceNoticeActiveGuardRule {
    pub limit: f64,
}

impl EvidenceNoticeRule for EvidenceNoticeActiveGuardRule {
    fn check(&self, item: &EvidenceNotice) -> Vec<String> {
        if item.active && (item.minimum > self.limit) {
            return vec![format!("minimum {} exceeds {} while active", item.minimum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceNoticeRequiredCodeRule {
    pub message: String,
}

impl EvidenceNoticeRule for EvidenceNoticeRequiredCodeRule {
    fn check(&self, item: &EvidenceNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceNoticeLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceNoticeRule for EvidenceNoticeLimitRangeRule {
    fn check(&self, item: &EvidenceNotice) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceNoticeMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EvidenceNoticeRule for EvidenceNoticeMinimumRangeRule {
    fn check(&self, item: &EvidenceNotice) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a evidence notice.
#[derive(Debug, Clone)]
pub struct EvidenceNoticeValidator {
    pub rules: Vec<Rc<dyn EvidenceNoticeRule>>,
}

impl EvidenceNoticeValidator {
    pub fn validate(&self, item: &EvidenceNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_notice_validator() -> EvidenceNoticeValidator {
    EvidenceNoticeValidator {
        rules: vec![
            Rc::new(EvidenceNoticeActiveGuardRule { limit: 3.5 }) as Rc<dyn EvidenceNoticeRule>,
            Rc::new(EvidenceNoticeRequiredCodeRule {
                message: "evidence notice needs an entry".to_string(),
            }),
            Rc::new(EvidenceNoticeLimitRangeRule { min: 80, max: 80 }),
            Rc::new(EvidenceNoticeMinimumRangeRule { min: 6.9, max: 1505.4 }),
        ],
    }
}
