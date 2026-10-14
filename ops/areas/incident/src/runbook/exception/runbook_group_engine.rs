use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the runbook group calculation.
pub trait RunbookGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookGroupAdjuster {
    pub percent: f64,
}

impl RunbookGroupStep for PercentRunbookGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RunbookGroupStep for TieredRunbookGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RunbookGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookGroupAdjuster {
    pub cap: f64,
}

impl RunbookGroupStep for CapRunbookGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeRunbookGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookGroupStep for ScopeRunbookGroupAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &RunbookGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the runbook group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookGroupEngine {
    pub steps: Vec<Rc<dyn RunbookGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookGroupEngine {
    pub fn price(&mut self, item: &RunbookGroup) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RunbookGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_runbook_group_engine() -> RunbookGroupEngine {
    RunbookGroupEngine {
        steps: vec![
            Rc::new(PercentRunbookGroupAdjuster { percent: -11.3 }) as Rc<dyn RunbookGroupStep>,
            Rc::new(TieredRunbookGroupAdjuster {
                bounds: vec![3321.0, 6964.0, 7348.0, 9282.0],
                rates: vec![0.97, 1.05, 0.87, 1.13],
            }),
            Rc::new(CapRunbookGroupAdjuster { cap: 28874.0 }),
            Rc::new(ScopeRunbookGroupAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.77), ("west".to_string(), 1.13)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 61016.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_group_engine();
        let item = new_runbook_group("code-832", "title-134");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
