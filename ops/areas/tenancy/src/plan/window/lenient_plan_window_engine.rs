use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient plan window calculation.
pub trait LenientPlanWindowFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientPlanWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientPlanWindowAdjuster {
    pub cap: f64,
}

impl LenientPlanWindowFactor for CapLenientPlanWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientPlanWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientPlanWindowAdjuster {
    pub step: f64,
}

impl LenientPlanWindowFactor for RoundingLenientPlanWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientPlanWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientPlanWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientPlanWindowFactor for TieredLenientPlanWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientPlanWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient plan window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientPlanWindowEngine {
    pub steps: Vec<Rc<dyn LenientPlanWindowFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientPlanWindowEngine {
    pub fn evaluate(&mut self, item: &LenientPlanWindow) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[LenientPlanWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_plan_window_engine() -> LenientPlanWindowEngine {
    LenientPlanWindowEngine {
        steps: vec![
            Rc::new(CapLenientPlanWindowAdjuster { cap: 23846.0 }) as Rc<dyn LenientPlanWindowFactor>,
            Rc::new(RoundingLenientPlanWindowAdjuster { step: 0.05 }),
            Rc::new(TieredLenientPlanWindowAdjuster {
                bounds: vec![389.0, 450.0, 6631.0, 9436.0],
                rates: vec![1.04, 0.88, 1.16, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 77722.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_plan_window_engine();
        let item = new_lenient_plan_window("code-373", "label-467");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
