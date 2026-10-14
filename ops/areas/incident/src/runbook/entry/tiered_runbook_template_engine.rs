use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tiered runbook template calculation.
pub trait TieredRunbookTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredRunbookTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredRunbookTemplateAdjuster {
    pub cap: f64,
}

impl TieredRunbookTemplateFactor for CapTieredRunbookTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredRunbookTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredRunbookTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredRunbookTemplateFactor for FlaggedTieredRunbookTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredRunbookTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredRunbookTemplateAdjuster {
    pub percent: f64,
}

impl TieredRunbookTemplateFactor for PercentTieredRunbookTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredRunbookTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered runbook template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredRunbookTemplateEngine {
    pub steps: Vec<Rc<dyn TieredRunbookTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredRunbookTemplateEngine {
    pub fn price(&mut self, item: &TieredRunbookTemplate) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredRunbookTemplateFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredRunbookTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_runbook_template_engine(flags: &flags::Registry) -> TieredRunbookTemplateEngine {
    TieredRunbookTemplateEngine {
        steps: vec![
            Rc::new(CapTieredRunbookTemplateAdjuster { cap: 39499.0 }) as Rc<dyn TieredRunbookTemplateFactor>,
            Rc::new(FlaggedTieredRunbookTemplateAdjuster {
                flags: flags.clone(),
                flag: "tiered-runbook-template-adaptive".to_string(),
                percent: 9.9,
            }),
            Rc::new(PercentTieredRunbookTemplateAdjuster { percent: 15.7 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 52292.4,
    }
}
