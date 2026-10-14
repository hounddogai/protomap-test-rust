use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal dashboard snapshot calculation.
pub trait SeasonalDashboardSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalDashboardSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalDashboardSnapshotAdjuster {
    pub minimum: f64,
}

impl SeasonalDashboardSnapshotStep for MinimumSeasonalDashboardSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalDashboardSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardSnapshotMaximumOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalDashboardSnapshotStep for SeasonalDashboardSnapshotMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalDashboardSnapshot, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalDashboardSnapshotAdjuster {
    pub step: f64,
}

impl SeasonalDashboardSnapshotStep for RoundingSeasonalDashboardSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalDashboardSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterSeasonalDashboardSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalDashboardSnapshotStep for ClusterSeasonalDashboardSnapshotAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &SeasonalDashboardSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalDashboardSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalDashboardSnapshotStep for TieredSeasonalDashboardSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalDashboardSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the seasonal dashboard snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalDashboardSnapshotEngine {
    pub steps: Vec<Rc<dyn SeasonalDashboardSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalDashboardSnapshotEngine {
    pub fn compute(&mut self, item: &SeasonalDashboardSnapshot) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalDashboardSnapshotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalDashboardSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_dashboard_snapshot_engine() -> SeasonalDashboardSnapshotEngine {
    SeasonalDashboardSnapshotEngine {
        steps: vec![
            Rc::new(MinimumSeasonalDashboardSnapshotAdjuster { minimum: 30.9 })
                as Rc<dyn SeasonalDashboardSnapshotStep>,
            Rc::new(SeasonalDashboardSnapshotMaximumOffsetAdjuster { weight: 0.38 }),
            Rc::new(RoundingSeasonalDashboardSnapshotAdjuster { step: 1.0 }),
            Rc::new(ClusterSeasonalDashboardSnapshotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.27), ("north".to_string(), 1.23)]),
            }),
            Rc::new(TieredSeasonalDashboardSnapshotAdjuster {
                bounds: vec![2741.0, 2944.0, 5944.0],
                rates: vec![1.18, 0.9, 0.93],
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 14171.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_dashboard_snapshot_engine();
        let item = new_seasonal_dashboard_snapshot("code-231", "label-564");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
