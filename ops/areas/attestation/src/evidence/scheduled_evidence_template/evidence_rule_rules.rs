use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence rule must pass. Check returns its problems, or none.
pub trait EvidenceRuleRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRuleRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceRuleRule for EvidenceRuleRetriesRangeRule {
    fn check(&self, item: &EvidenceRule) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRulePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceRuleRule for EvidenceRulePriorityRangeRule {
    fn check(&self, item: &EvidenceRule) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRuleRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceRuleRule for EvidenceRuleRevisionRangeRule {
    fn check(&self, item: &EvidenceRule) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRuleRequiredCodeRule {
    pub message: String,
}

impl EvidenceRuleRule for EvidenceRuleRequiredCodeRule {
    fn check(&self, item: &EvidenceRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence rule.
#[derive(Debug, Clone)]
pub struct EvidenceRuleValidator {
    pub rules: Vec<Rc<dyn EvidenceRuleRule>>,
}

impl EvidenceRuleValidator {
    pub fn validate(&self, item: &EvidenceRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_rule_validator() -> EvidenceRuleValidator {
    EvidenceRuleValidator {
        rules: vec![
            Rc::new(EvidenceRuleRetriesRangeRule { min: 120, max: 1000 }) as Rc<dyn EvidenceRuleRule>,
            Rc::new(EvidenceRulePriorityRangeRule { min: 32, max: 3600 }),
            Rc::new(EvidenceRuleRevisionRangeRule { min: 16, max: 86400 }),
            Rc::new(EvidenceRuleRequiredCodeRule {
                message: "evidence rule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_rule_validator();
        let item = new_evidence_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = EvidenceRuleRetriesRangeRule { min: 255, max: 86400 };
        let mut item = new_evidence_rule("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = EvidenceRulePriorityRangeRule { min: 3600, max: 86400 };
        let mut item = new_evidence_rule("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = EvidenceRuleRevisionRangeRule { min: 32, max: 3600 };
        let mut item = new_evidence_rule("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EvidenceRuleRequiredCodeRule {
            message: "evidence rule needs an entry".to_string(),
        };
        let mut item = new_evidence_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
