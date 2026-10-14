use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the usage cycle calculation.
pub trait UsageCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageCycleEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl UsageCycleFactor for UsageCycleEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &UsageCycle, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageCycleAdjuster {
    pub cap: f64,
}

impl UsageCycleFactor for CapUsageCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentUsageCycleAdjuster {
    pub percent: f64,
}

impl UsageCycleFactor for PercentUsageCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &UsageCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageCycleAdjuster {
    pub minimum: f64,
}

impl UsageCycleFactor for MinimumUsageCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingUsageCycleAdjuster {
    pub step: f64,
}

impl UsageCycleFactor for RoundingUsageCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &UsageCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl UsageCycleFactor for TieredUsageCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &UsageCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the usage cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageCycleEngine {
    pub steps: Vec<Rc<dyn UsageCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageCycleEngine {
    pub fn evaluate(&mut self, item: &UsageCycle) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[UsageCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_usage_cycle_engine() -> UsageCycleEngine {
    UsageCycleEngine {
        steps: vec![
            Rc::new(UsageCycleEndsAtOffsetAdjuster { weight: 0.43 }) as Rc<dyn UsageCycleFactor>,
            Rc::new(CapUsageCycleAdjuster { cap: 30837.0 }),
            Rc::new(PercentUsageCycleAdjuster { percent: -13.3 }),
            Rc::new(MinimumUsageCycleAdjuster { minimum: 24.0 }),
            Rc::new(RoundingUsageCycleAdjuster { step: 0.25 }),
            Rc::new(TieredUsageCycleAdjuster {
                bounds: vec![4259.0, 4816.0],
                rates: vec![1.03, 0.81],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 74386.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_cycle_engine();
        let item = new_usage_cycle("code-900", "title-394");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
