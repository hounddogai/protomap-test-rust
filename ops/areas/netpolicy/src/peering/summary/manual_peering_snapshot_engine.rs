use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual peering snapshot calculation.
pub trait ManualPeeringSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualPeeringSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualPeeringSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualPeeringSnapshotFactor for TieredManualPeeringSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualPeeringSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelManualPeeringSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualPeeringSnapshotFactor for ChannelManualPeeringSnapshotAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &ManualPeeringSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualPeeringSnapshotAdjuster {
    pub cap: f64,
}

impl ManualPeeringSnapshotFactor for CapManualPeeringSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualPeeringSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualPeeringSnapshotAdjuster {
    pub percent: f64,
}

impl ManualPeeringSnapshotFactor for PercentManualPeeringSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualPeeringSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualPeeringSnapshotAdjuster {
    pub step: f64,
}

impl ManualPeeringSnapshotFactor for RoundingManualPeeringSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualPeeringSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual peering snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualPeeringSnapshotEngine {
    pub steps: Vec<Rc<dyn ManualPeeringSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualPeeringSnapshotEngine {
    pub fn run(&mut self, item: &ManualPeeringSnapshot) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualPeeringSnapshotFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualPeeringSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_peering_snapshot_engine() -> ManualPeeringSnapshotEngine {
    ManualPeeringSnapshotEngine {
        steps: vec![
            Rc::new(TieredManualPeeringSnapshotAdjuster {
                bounds: vec![3079.0, 8777.0, 9558.0],
                rates: vec![1.1, 0.88, 1.01],
            }) as Rc<dyn ManualPeeringSnapshotFactor>,
            Rc::new(ChannelManualPeeringSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.28),
                    ("retail".to_string(), 1.31),
                    ("south".to_string(), 1.3),
                ]),
            }),
            Rc::new(CapManualPeeringSnapshotAdjuster { cap: 3671.0 }),
            Rc::new(PercentManualPeeringSnapshotAdjuster { percent: -16.7 }),
            Rc::new(RoundingManualPeeringSnapshotAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 40412.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_peering_snapshot_engine();
        let item = new_manual_peering_snapshot("code-153", "label-914");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
