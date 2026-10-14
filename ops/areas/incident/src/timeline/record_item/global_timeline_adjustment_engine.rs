use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global timeline adjustment calculation.
pub trait GlobalTimelineAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalTimelineAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceGlobalTimelineAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalTimelineAdjustmentFactor for NamespaceGlobalTimelineAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &GlobalTimelineAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalTimelineAdjustmentAdjuster {
    pub percent: f64,
}

impl GlobalTimelineAdjustmentFactor for PercentGlobalTimelineAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalTimelineAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalTimelineAdjustmentAdjuster {
    pub minimum: f64,
}

impl GlobalTimelineAdjustmentFactor for MinimumGlobalTimelineAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalTimelineAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineAdjustmentTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl GlobalTimelineAdjustmentFactor for GlobalTimelineAdjustmentTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalTimelineAdjustment, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalTimelineAdjustmentAdjuster {
    pub step: f64,
}

impl GlobalTimelineAdjustmentFactor for RoundingGlobalTimelineAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalTimelineAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalTimelineAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalTimelineAdjustmentFactor for TieredGlobalTimelineAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalTimelineAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the global timeline adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalTimelineAdjustmentEngine {
    pub steps: Vec<Rc<dyn GlobalTimelineAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalTimelineAdjustmentEngine {
    pub fn price(&mut self, item: &GlobalTimelineAdjustment) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalTimelineAdjustmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalTimelineAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_timeline_adjustment_engine() -> GlobalTimelineAdjustmentEngine {
    GlobalTimelineAdjustmentEngine {
        steps: vec![
            Rc::new(NamespaceGlobalTimelineAdjustmentAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.78), ("online".to_string(), 1.22)]),
            }) as Rc<dyn GlobalTimelineAdjustmentFactor>,
            Rc::new(PercentGlobalTimelineAdjustmentAdjuster { percent: 22.0 }),
            Rc::new(MinimumGlobalTimelineAdjustmentAdjuster { minimum: 35.6 }),
            Rc::new(GlobalTimelineAdjustmentTtlSecondsOffsetAdjuster { weight: 1.88 }),
            Rc::new(RoundingGlobalTimelineAdjustmentAdjuster { step: 1.0 }),
            Rc::new(TieredGlobalTimelineAdjustmentAdjuster {
                bounds: vec![309.0, 2234.0, 3779.0, 7436.0],
                rates: vec![1.08, 1.1, 0.99, 0.96],
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 10992.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_timeline_adjustment_engine();
        let item = new_global_timeline_adjustment("code-812", "title-680");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
