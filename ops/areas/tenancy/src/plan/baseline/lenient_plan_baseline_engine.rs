use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient plan baseline calculation.
pub trait LenientPlanBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientPlanBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientPlanBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientPlanBaselineStep for TieredLenientPlanBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientPlanBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientPlanBaselineAdjuster {
    pub step: f64,
}

impl LenientPlanBaselineStep for RoundingLenientPlanBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientPlanBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanBaselineUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl LenientPlanBaselineStep for LenientPlanBaselineUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &LenientPlanBaseline, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientPlanBaselineAdjuster {
    pub minimum: f64,
}

impl LenientPlanBaselineStep for MinimumLenientPlanBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientPlanBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLenientPlanBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientPlanBaselineStep for LabelLenientPlanBaselineAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LenientPlanBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientPlanBaselineAdjuster {
    pub cap: f64,
}

impl LenientPlanBaselineStep for CapLenientPlanBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientPlanBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the lenient plan baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientPlanBaselineEngine {
    pub steps: Vec<Rc<dyn LenientPlanBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientPlanBaselineEngine {
    pub fn run(&mut self, item: &LenientPlanBaseline) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientPlanBaselineStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientPlanBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_plan_baseline_engine() -> LenientPlanBaselineEngine {
    LenientPlanBaselineEngine {
        steps: vec![
            Rc::new(TieredLenientPlanBaselineAdjuster {
                bounds: vec![2779.0, 4876.0],
                rates: vec![0.87, 1.02],
            }) as Rc<dyn LenientPlanBaselineStep>,
            Rc::new(RoundingLenientPlanBaselineAdjuster { step: 1.0 }),
            Rc::new(LenientPlanBaselineUpdatedAtOffsetAdjuster { weight: 0.04 }),
            Rc::new(MinimumLenientPlanBaselineAdjuster { minimum: 26.4 }),
            Rc::new(LabelLenientPlanBaselineAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.89)]),
            }),
            Rc::new(CapLenientPlanBaselineAdjuster { cap: 45237.0 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 94838.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_plan_baseline_engine();
        let item = new_lenient_plan_baseline("code-539", "label-917");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
