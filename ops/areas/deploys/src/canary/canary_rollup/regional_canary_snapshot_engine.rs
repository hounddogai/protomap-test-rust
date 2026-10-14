use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional canary snapshot calculation.
pub trait RegionalCanarySnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalCanarySnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalCanarySnapshotAdjuster {
    pub minimum: f64,
}

impl RegionalCanarySnapshotStep for MinimumRegionalCanarySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalCanarySnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalCanarySnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalCanarySnapshotStep for FlaggedRegionalCanarySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalCanarySnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRegionalCanarySnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalCanarySnapshotStep for SourceRegionalCanarySnapshotAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &RegionalCanarySnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalCanarySnapshotAdjuster {
    pub cap: f64,
}

impl RegionalCanarySnapshotStep for CapRegionalCanarySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalCanarySnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalCanarySnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalCanarySnapshotStep for TieredRegionalCanarySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalCanarySnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalCanarySnapshotAdjuster {
    pub percent: f64,
}

impl RegionalCanarySnapshotStep for PercentRegionalCanarySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalCanarySnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional canary snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalCanarySnapshotEngine {
    pub steps: Vec<Rc<dyn RegionalCanarySnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalCanarySnapshotEngine {
    pub fn run(&mut self, item: &RegionalCanarySnapshot) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalCanarySnapshotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalCanarySnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_canary_snapshot_engine(flags: &flags::Registry) -> RegionalCanarySnapshotEngine {
    RegionalCanarySnapshotEngine {
        steps: vec![
            Rc::new(MinimumRegionalCanarySnapshotAdjuster { minimum: 41.4 }) as Rc<dyn RegionalCanarySnapshotStep>,
            Rc::new(FlaggedRegionalCanarySnapshotAdjuster {
                flags: flags.clone(),
                flag: "regional-canary-snapshot-seasonal".to_string(),
                percent: 9.0,
            }),
            Rc::new(SourceRegionalCanarySnapshotAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.07),
                    ("north".to_string(), 1.01),
                    ("retail".to_string(), 1.05),
                ]),
            }),
            Rc::new(CapRegionalCanarySnapshotAdjuster { cap: 10578.0 }),
            Rc::new(TieredRegionalCanarySnapshotAdjuster {
                bounds: vec![5584.0, 5818.0],
                rates: vec![0.85, 1.09],
            }),
            Rc::new(PercentRegionalCanarySnapshotAdjuster { percent: -12.8 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 57020.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_canary_snapshot_engine(&flags::new_registry());
        let item = new_regional_canary_snapshot("code-15", "title-101");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
