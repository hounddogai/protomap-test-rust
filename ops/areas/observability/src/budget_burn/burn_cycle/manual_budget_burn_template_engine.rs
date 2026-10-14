use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the manual budget burn template calculation.
pub trait ManualBudgetBurnTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualBudgetBurnTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterManualBudgetBurnTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualBudgetBurnTemplateFactor for ClusterManualBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &ManualBudgetBurnTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnTemplateQuantityOffsetAdjuster {
    pub weight: f64,
}

impl ManualBudgetBurnTemplateFactor for ManualBudgetBurnTemplateQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &ManualBudgetBurnTemplate, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualBudgetBurnTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualBudgetBurnTemplateFactor for TieredManualBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualBudgetBurnTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual budget burn template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualBudgetBurnTemplateEngine {
    pub steps: Vec<Rc<dyn ManualBudgetBurnTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualBudgetBurnTemplateEngine {
    pub fn run(&mut self, item: &ManualBudgetBurnTemplate) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualBudgetBurnTemplateFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualBudgetBurnTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_budget_burn_template_engine() -> ManualBudgetBurnTemplateEngine {
    ManualBudgetBurnTemplateEngine {
        steps: vec![
            Rc::new(ClusterManualBudgetBurnTemplateAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.2),
                    ("north".to_string(), 0.85),
                    ("south".to_string(), 1.39),
                ]),
            }) as Rc<dyn ManualBudgetBurnTemplateFactor>,
            Rc::new(ManualBudgetBurnTemplateQuantityOffsetAdjuster { weight: 0.23 }),
            Rc::new(TieredManualBudgetBurnTemplateAdjuster {
                bounds: vec![1594.0, 7010.0, 7520.0],
                rates: vec![1.08, 1.12, 1.14],
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 62521.2,
    }
}
