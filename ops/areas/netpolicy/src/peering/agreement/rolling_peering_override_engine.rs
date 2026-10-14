use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling peering override calculation.
pub trait RollingPeeringOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPeeringOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingPeeringOverrideAdjuster {
    pub cap: f64,
}

impl RollingPeeringOverrideAdjuster for CapRollingPeeringOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingPeeringOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingPeeringOverrideAdjuster {
    pub percent: f64,
}

impl RollingPeeringOverrideAdjuster for PercentRollingPeeringOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingPeeringOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingPeeringOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingPeeringOverrideAdjuster for FlaggedRollingPeeringOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingPeeringOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingPeeringOverrideAdjuster {
    pub step: f64,
}

impl RollingPeeringOverrideAdjuster for RoundingRollingPeeringOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingPeeringOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingPeeringOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingPeeringOverrideAdjuster for TieredRollingPeeringOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingPeeringOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringOverrideTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl RollingPeeringOverrideAdjuster for RollingPeeringOverrideTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &RollingPeeringOverride, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

/// Runs the rolling peering override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPeeringOverrideEngine {
    pub steps: Vec<Rc<dyn RollingPeeringOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPeeringOverrideEngine {
    pub fn evaluate(&mut self, item: &RollingPeeringOverride) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingPeeringOverrideAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingPeeringOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_peering_override_engine(flags: &flags::Registry) -> RollingPeeringOverrideEngine {
    RollingPeeringOverrideEngine {
        steps: vec![
            Rc::new(CapRollingPeeringOverrideAdjuster { cap: 22186.0 }) as Rc<dyn RollingPeeringOverrideAdjuster>,
            Rc::new(PercentRollingPeeringOverrideAdjuster { percent: -2.9 }),
            Rc::new(FlaggedRollingPeeringOverrideAdjuster {
                flags: flags.clone(),
                flag: "rolling-peering-override-default".to_string(),
                percent: -3.0,
            }),
            Rc::new(RoundingRollingPeeringOverrideAdjuster { step: 0.1 }),
            Rc::new(TieredRollingPeeringOverrideAdjuster {
                bounds: vec![317.0, 3381.0, 3890.0],
                rates: vec![0.85, 0.95, 0.88],
            }),
            Rc::new(RollingPeeringOverrideTtlSecondsOffsetAdjuster { weight: 1.58 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 51089.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_peering_override_engine(&flags::new_registry());
        let item = new_rolling_peering_override("code-180", "name-876");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
