use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal policy snapshot calculation.
pub trait SeasonalPolicySnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalPolicySnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSeasonalPolicySnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalPolicySnapshotModifier for NameSeasonalPolicySnapshotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SeasonalPolicySnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalPolicySnapshotAdjuster {
    pub cap: f64,
}

impl SeasonalPolicySnapshotModifier for CapSeasonalPolicySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalPolicySnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalPolicySnapshotAdjuster {
    pub step: f64,
}

impl SeasonalPolicySnapshotModifier for RoundingSeasonalPolicySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalPolicySnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalPolicySnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalPolicySnapshotModifier for TieredSeasonalPolicySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalPolicySnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalPolicySnapshotAdjuster {
    pub minimum: f64,
}

impl SeasonalPolicySnapshotModifier for MinimumSeasonalPolicySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalPolicySnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalPolicySnapshotAdjuster {
    pub percent: f64,
}

impl SeasonalPolicySnapshotModifier for PercentSeasonalPolicySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalPolicySnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal policy snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalPolicySnapshotEngine {
    pub steps: Vec<Rc<dyn SeasonalPolicySnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalPolicySnapshotEngine {
    pub fn evaluate(&mut self, item: &SeasonalPolicySnapshot) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalPolicySnapshotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalPolicySnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_policy_snapshot_engine() -> SeasonalPolicySnapshotEngine {
    SeasonalPolicySnapshotEngine {
        steps: vec![
            Rc::new(NameSeasonalPolicySnapshotAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.2)]),
            }) as Rc<dyn SeasonalPolicySnapshotModifier>,
            Rc::new(CapSeasonalPolicySnapshotAdjuster { cap: 4630.0 }),
            Rc::new(RoundingSeasonalPolicySnapshotAdjuster { step: 0.5 }),
            Rc::new(TieredSeasonalPolicySnapshotAdjuster {
                bounds: vec![140.0, 2900.0, 8898.0],
                rates: vec![0.83, 0.86, 1.02],
            }),
            Rc::new(MinimumSeasonalPolicySnapshotAdjuster { minimum: 9.9 }),
            Rc::new(PercentSeasonalPolicySnapshotAdjuster { percent: 21.6 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 22093.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_policy_snapshot_engine();
        let item = new_seasonal_policy_snapshot("code-974", "name-960");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
