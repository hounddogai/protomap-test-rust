use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tiered credential rule calculation.
pub trait TieredCredentialRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredCredentialRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleTieredCredentialRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredCredentialRuleFactor for TitleTieredCredentialRuleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &TieredCredentialRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredCredentialRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredCredentialRuleFactor for FlaggedTieredCredentialRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredCredentialRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredCredentialRuleAdjuster {
    pub percent: f64,
}

impl TieredCredentialRuleFactor for PercentTieredCredentialRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredCredentialRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered credential rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredCredentialRuleEngine {
    pub steps: Vec<Rc<dyn TieredCredentialRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredCredentialRuleEngine {
    pub fn evaluate(&mut self, item: &TieredCredentialRule) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredCredentialRuleFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[TieredCredentialRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_credential_rule_engine(flags: &flags::Registry) -> TieredCredentialRuleEngine {
    TieredCredentialRuleEngine {
        steps: vec![
            Rc::new(TitleTieredCredentialRuleAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.35)]),
            }) as Rc<dyn TieredCredentialRuleFactor>,
            Rc::new(FlaggedTieredCredentialRuleAdjuster {
                flags: flags.clone(),
                flag: "tiered-credential-rule-seasonal".to_string(),
                percent: 1.5,
            }),
            Rc::new(PercentTieredCredentialRuleAdjuster { percent: 9.7 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 12383.3,
    }
}
