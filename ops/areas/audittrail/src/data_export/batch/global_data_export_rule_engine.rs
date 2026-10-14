use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global data export rule calculation.
pub trait GlobalDataExportRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalDataExportRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalDataExportRuleAdjuster {
    pub cap: f64,
}

impl GlobalDataExportRuleStep for CapGlobalDataExportRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalDataExportRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalDataExportRuleAdjuster {
    pub percent: f64,
}

impl GlobalDataExportRuleStep for PercentGlobalDataExportRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalDataExportRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalDataExportRuleAdjuster {
    pub step: f64,
}

impl GlobalDataExportRuleStep for RoundingGlobalDataExportRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalDataExportRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelGlobalDataExportRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalDataExportRuleStep for LabelGlobalDataExportRuleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &GlobalDataExportRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalDataExportRuleAdjuster {
    pub minimum: f64,
}

impl GlobalDataExportRuleStep for MinimumGlobalDataExportRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalDataExportRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalDataExportRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalDataExportRuleStep for TieredGlobalDataExportRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalDataExportRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the global data export rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalDataExportRuleEngine {
    pub steps: Vec<Rc<dyn GlobalDataExportRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalDataExportRuleEngine {
    pub fn compute(&mut self, item: &GlobalDataExportRule) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalDataExportRuleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalDataExportRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_data_export_rule_engine() -> GlobalDataExportRuleEngine {
    GlobalDataExportRuleEngine {
        steps: vec![
            Rc::new(CapGlobalDataExportRuleAdjuster { cap: 24097.0 }) as Rc<dyn GlobalDataExportRuleStep>,
            Rc::new(PercentGlobalDataExportRuleAdjuster { percent: -3.4 }),
            Rc::new(RoundingGlobalDataExportRuleAdjuster { step: 0.05 }),
            Rc::new(LabelGlobalDataExportRuleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.03), ("north".to_string(), 0.97)]),
            }),
            Rc::new(MinimumGlobalDataExportRuleAdjuster { minimum: 40.7 }),
            Rc::new(TieredGlobalDataExportRuleAdjuster {
                bounds: vec![3053.0, 4041.0, 4123.0, 5709.0],
                rates: vec![1.08, 0.92, 1.07, 1.1],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 23780.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_data_export_rule_engine();
        let item = new_global_data_export_rule("code-701", "label-817");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
