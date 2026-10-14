use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the grant summary calculation.
pub trait GrantSummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryGrantSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantSummaryFactor for CategoryGrantSummaryAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &GrantSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantSummaryAdjuster {
    pub minimum: f64,
}

impl GrantSummaryFactor for MinimumGrantSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GrantSummaryFactor for TieredGrantSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GrantSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSummaryThresholdOffsetAdjuster {
    pub weight: f64,
}

impl GrantSummaryFactor for GrantSummaryThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &GrantSummary, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGrantSummaryAdjuster {
    pub cap: f64,
}

impl GrantSummaryFactor for CapGrantSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GrantSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGrantSummaryAdjuster {
    pub step: f64,
}

impl GrantSummaryFactor for RoundingGrantSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GrantSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the grant summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantSummaryEngine {
    pub steps: Vec<Rc<dyn GrantSummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantSummaryEngine {
    pub fn price(&mut self, item: &GrantSummary) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GrantSummaryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GrantSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_grant_summary_engine() -> GrantSummaryEngine {
    GrantSummaryEngine {
        steps: vec![
            Rc::new(CategoryGrantSummaryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.01)]),
            }) as Rc<dyn GrantSummaryFactor>,
            Rc::new(MinimumGrantSummaryAdjuster { minimum: 23.4 }),
            Rc::new(TieredGrantSummaryAdjuster {
                bounds: vec![2333.0, 2796.0, 9496.0],
                rates: vec![0.92, 0.9, 1.11],
            }),
            Rc::new(GrantSummaryThresholdOffsetAdjuster { weight: 1.81 }),
            Rc::new(CapGrantSummaryAdjuster { cap: 5746.0 }),
            Rc::new(RoundingGrantSummaryAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 82920.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_summary_engine();
        let item = new_grant_summary("code-962", "label-254");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
