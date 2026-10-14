use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling timeline policy calculation.
pub trait RollingTimelinePolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingTimelinePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingTimelinePolicyAdjuster {
    pub minimum: f64,
}

impl RollingTimelinePolicyModifier for MinimumRollingTimelinePolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingTimelinePolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingTimelinePolicyAdjuster {
    pub percent: f64,
}

impl RollingTimelinePolicyModifier for PercentRollingTimelinePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingTimelinePolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelinePolicyQuantityOffsetAdjuster {
    pub weight: f64,
}

impl RollingTimelinePolicyModifier for RollingTimelinePolicyQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &RollingTimelinePolicy, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingTimelinePolicyAdjuster {
    pub step: f64,
}

impl RollingTimelinePolicyModifier for RoundingRollingTimelinePolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingTimelinePolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingTimelinePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingTimelinePolicyModifier for TieredRollingTimelinePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingTimelinePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingTimelinePolicyAdjuster {
    pub cap: f64,
}

impl RollingTimelinePolicyModifier for CapRollingTimelinePolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingTimelinePolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling timeline policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingTimelinePolicyEngine {
    pub steps: Vec<Rc<dyn RollingTimelinePolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingTimelinePolicyEngine {
    pub fn price(&mut self, item: &RollingTimelinePolicy) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[RollingTimelinePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_timeline_policy_engine() -> RollingTimelinePolicyEngine {
    RollingTimelinePolicyEngine {
        steps: vec![
            Rc::new(MinimumRollingTimelinePolicyAdjuster { minimum: 22.3 }) as Rc<dyn RollingTimelinePolicyModifier>,
            Rc::new(PercentRollingTimelinePolicyAdjuster { percent: 4.6 }),
            Rc::new(RollingTimelinePolicyQuantityOffsetAdjuster { weight: 1.08 }),
            Rc::new(RoundingRollingTimelinePolicyAdjuster { step: 1.0 }),
            Rc::new(TieredRollingTimelinePolicyAdjuster {
                bounds: vec![3488.0, 6356.0, 8283.0],
                rates: vec![1.1, 1.19, 1.03],
            }),
            Rc::new(CapRollingTimelinePolicyAdjuster { cap: 10959.0 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 15932.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_timeline_policy_engine();
        let item = new_rolling_timeline_policy("code-106", "title-672");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
