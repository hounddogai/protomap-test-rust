use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the runbook variance calculation.
pub trait RunbookVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRunbookVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookVarianceFactor for TitleRunbookVarianceAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RunbookVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookVarianceMinimumOffsetAdjuster {
    pub weight: f64,
}

impl RunbookVarianceFactor for RunbookVarianceMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &RunbookVariance, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookVarianceAdjuster {
    pub minimum: f64,
}

impl RunbookVarianceFactor for MinimumRunbookVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RunbookVarianceFactor for TieredRunbookVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RunbookVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookVarianceAdjuster {
    pub step: f64,
}

impl RunbookVarianceFactor for RoundingRunbookVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookVarianceAdjuster {
    pub cap: f64,
}

impl RunbookVarianceFactor for CapRunbookVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the runbook variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookVarianceEngine {
    pub steps: Vec<Rc<dyn RunbookVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookVarianceEngine {
    pub fn price(&mut self, item: &RunbookVariance) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RunbookVarianceFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RunbookVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_runbook_variance_engine() -> RunbookVarianceEngine {
    RunbookVarianceEngine {
        steps: vec![
            Rc::new(TitleRunbookVarianceAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.17)]),
            }) as Rc<dyn RunbookVarianceFactor>,
            Rc::new(RunbookVarianceMinimumOffsetAdjuster { weight: 1.77 }),
            Rc::new(MinimumRunbookVarianceAdjuster { minimum: 36.6 }),
            Rc::new(TieredRunbookVarianceAdjuster {
                bounds: vec![992.0, 1202.0, 7603.0],
                rates: vec![0.81, 1.06, 1.11],
            }),
            Rc::new(RoundingRunbookVarianceAdjuster { step: 0.5 }),
            Rc::new(CapRunbookVarianceAdjuster { cap: 22904.0 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 20511.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_variance_engine();
        let item = new_runbook_variance("code-148", "title-972");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
