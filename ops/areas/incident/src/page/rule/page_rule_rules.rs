use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a page rule must pass. Check returns its problems, or none.
pub trait PageRuleRule: std::fmt::Debug {
    fn check(&self, item: &PageRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRuleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageRuleRule for PageRuleScoreRangeRule {
    fn check(&self, item: &PageRule) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRuleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageRuleRule for PageRuleDiscountRangeRule {
    fn check(&self, item: &PageRule) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRuleRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageRuleRule for PageRuleRateRangeRule {
    fn check(&self, item: &PageRule) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRuleBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageRuleRule for PageRuleBurstRangeRule {
    fn check(&self, item: &PageRule) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PageRuleRule for PageRuleAllowedCodeRule {
    fn check(&self, item: &PageRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRuleRequiredContactEmailRule {
    pub message: String,
}

impl PageRuleRule for PageRuleRequiredContactEmailRule {
    fn check(&self, item: &PageRule) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a page rule.
#[derive(Debug, Clone)]
pub struct PageRuleValidator {
    pub rules: Vec<Rc<dyn PageRuleRule>>,
}

impl PageRuleValidator {
    pub fn validate(&self, item: &PageRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_rule_validator() -> PageRuleValidator {
    PageRuleValidator {
        rules: vec![
            Rc::new(PageRuleScoreRangeRule { min: 7.7, max: 4454.1 }) as Rc<dyn PageRuleRule>,
            Rc::new(PageRuleDiscountRangeRule { min: 6.0, max: 752.2 }),
            Rc::new(PageRuleRateRangeRule { min: 0.1, max: 701.5 }),
            Rc::new(PageRuleBurstRangeRule { min: 64, max: 3600 }),
            Rc::new(PageRuleAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(PageRuleRequiredContactEmailRule {
                message: "page rule needs a value".to_string(),
            }),
        ],
    }
}
