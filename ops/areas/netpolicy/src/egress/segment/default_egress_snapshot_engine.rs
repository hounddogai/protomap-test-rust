use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default egress snapshot calculation.
pub trait DefaultEgressSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultEgressSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressSnapshotTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl DefaultEgressSnapshotStep for DefaultEgressSnapshotTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultEgressSnapshot, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultEgressSnapshotAdjuster {
    pub step: f64,
}

impl DefaultEgressSnapshotStep for RoundingDefaultEgressSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultEgressSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultEgressSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultEgressSnapshotStep for TieredDefaultEgressSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultEgressSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultEgressSnapshotAdjuster {
    pub minimum: f64,
}

impl DefaultEgressSnapshotStep for MinimumDefaultEgressSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultEgressSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultEgressSnapshotAdjuster {
    pub percent: f64,
}

impl DefaultEgressSnapshotStep for PercentDefaultEgressSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultEgressSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitDefaultEgressSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultEgressSnapshotStep for UnitDefaultEgressSnapshotAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &DefaultEgressSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default egress snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultEgressSnapshotEngine {
    pub steps: Vec<Rc<dyn DefaultEgressSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultEgressSnapshotEngine {
    pub fn evaluate(&mut self, item: &DefaultEgressSnapshot) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultEgressSnapshotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultEgressSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_egress_snapshot_engine() -> DefaultEgressSnapshotEngine {
    DefaultEgressSnapshotEngine {
        steps: vec![
            Rc::new(DefaultEgressSnapshotTtlSecondsOffsetAdjuster { weight: 0.99 })
                as Rc<dyn DefaultEgressSnapshotStep>,
            Rc::new(RoundingDefaultEgressSnapshotAdjuster { step: 0.5 }),
            Rc::new(TieredDefaultEgressSnapshotAdjuster {
                bounds: vec![2031.0, 2050.0, 7726.0],
                rates: vec![0.98, 0.99, 1.0],
            }),
            Rc::new(MinimumDefaultEgressSnapshotAdjuster { minimum: 45.1 }),
            Rc::new(PercentDefaultEgressSnapshotAdjuster { percent: -9.1 }),
            Rc::new(UnitDefaultEgressSnapshotAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.78), ("south".to_string(), 1.1)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 31670.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_egress_snapshot_engine();
        let item = new_default_egress_snapshot("code-266", "title-13");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
