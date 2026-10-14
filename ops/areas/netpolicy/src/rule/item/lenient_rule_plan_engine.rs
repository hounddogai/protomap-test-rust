use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient rule plan calculation.
pub trait LenientRulePlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientRulePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientRulePlanAdjuster {
    pub percent: f64,
}

impl LenientRulePlanAdjuster for PercentLenientRulePlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientRulePlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRulePlanRateOffsetAdjuster {
    pub weight: f64,
}

impl LenientRulePlanAdjuster for LenientRulePlanRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &LenientRulePlan, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientRulePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientRulePlanAdjuster for TieredLenientRulePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientRulePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientRulePlanAdjuster {
    pub step: f64,
}

impl LenientRulePlanAdjuster for RoundingLenientRulePlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientRulePlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the lenient rule plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientRulePlanEngine {
    pub steps: Vec<Rc<dyn LenientRulePlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientRulePlanEngine {
    pub fn compute(&mut self, item: &LenientRulePlan) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[LenientRulePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_rule_plan_engine() -> LenientRulePlanEngine {
    LenientRulePlanEngine {
        steps: vec![
            Rc::new(PercentLenientRulePlanAdjuster { percent: -18.9 }) as Rc<dyn LenientRulePlanAdjuster>,
            Rc::new(LenientRulePlanRateOffsetAdjuster { weight: 0.81 }),
            Rc::new(TieredLenientRulePlanAdjuster {
                bounds: vec![5212.0, 5670.0, 8888.0],
                rates: vec![0.84, 0.93, 1.09],
            }),
            Rc::new(RoundingLenientRulePlanAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 23094.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_rule_plan_engine();
        let item = new_lenient_rule_plan("code-447", "label-523");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
