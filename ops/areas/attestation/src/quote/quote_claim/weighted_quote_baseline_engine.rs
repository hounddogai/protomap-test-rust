use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted quote baseline calculation.
pub trait WeightedQuoteBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedQuoteBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedQuoteBaselineAdjuster {
    pub step: f64,
}

impl WeightedQuoteBaselineAdjuster for RoundingWeightedQuoteBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedQuoteBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionWeightedQuoteBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedQuoteBaselineAdjuster for RegionWeightedQuoteBaselineAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &WeightedQuoteBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBaselineBurstOffsetAdjuster {
    pub weight: f64,
}

impl WeightedQuoteBaselineAdjuster for WeightedQuoteBaselineBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedQuoteBaseline, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedQuoteBaselineAdjuster {
    pub cap: f64,
}

impl WeightedQuoteBaselineAdjuster for CapWeightedQuoteBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedQuoteBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the weighted quote baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedQuoteBaselineEngine {
    pub steps: Vec<Rc<dyn WeightedQuoteBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedQuoteBaselineEngine {
    pub fn evaluate(&mut self, item: &WeightedQuoteBaseline) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedQuoteBaselineAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedQuoteBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_quote_baseline_engine() -> WeightedQuoteBaselineEngine {
    WeightedQuoteBaselineEngine {
        steps: vec![
            Rc::new(RoundingWeightedQuoteBaselineAdjuster { step: 5.0 }) as Rc<dyn WeightedQuoteBaselineAdjuster>,
            Rc::new(RegionWeightedQuoteBaselineAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.78), ("west".to_string(), 0.95)]),
            }),
            Rc::new(WeightedQuoteBaselineBurstOffsetAdjuster { weight: 1.61 }),
            Rc::new(CapWeightedQuoteBaselineAdjuster { cap: 997.0 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 25741.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_quote_baseline_engine();
        let item = new_weighted_quote_baseline("code-557", "name-323");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
