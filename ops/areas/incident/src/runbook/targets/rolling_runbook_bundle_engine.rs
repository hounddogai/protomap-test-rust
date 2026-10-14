use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling runbook bundle calculation.
pub trait RollingRunbookBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingRunbookBundle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingRunbookBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingRunbookBundleAdjuster for FlaggedRollingRunbookBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingRunbookBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRollingRunbookBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingRunbookBundleAdjuster for SegmentRollingRunbookBundleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RollingRunbookBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingRunbookBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingRunbookBundleAdjuster for TieredRollingRunbookBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingRunbookBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingRunbookBundleAdjuster {
    pub cap: f64,
}

impl RollingRunbookBundleAdjuster for CapRollingRunbookBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingRunbookBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingRunbookBundleAdjuster {
    pub step: f64,
}

impl RollingRunbookBundleAdjuster for RoundingRollingRunbookBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingRunbookBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingRunbookBundleAdjuster {
    pub percent: f64,
}

impl RollingRunbookBundleAdjuster for PercentRollingRunbookBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingRunbookBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling runbook bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingRunbookBundleEngine {
    pub steps: Vec<Rc<dyn RollingRunbookBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingRunbookBundleEngine {
    pub fn price(&mut self, item: &RollingRunbookBundle) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingRunbookBundleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingRunbookBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_runbook_bundle_engine(flags: &flags::Registry) -> RollingRunbookBundleEngine {
    RollingRunbookBundleEngine {
        steps: vec![
            Rc::new(FlaggedRollingRunbookBundleAdjuster {
                flags: flags.clone(),
                flag: "rolling-runbook-bundle-default".to_string(),
                percent: -0.5,
            }) as Rc<dyn RollingRunbookBundleAdjuster>,
            Rc::new(SegmentRollingRunbookBundleAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.09),
                    ("north".to_string(), 1.25),
                    ("west".to_string(), 0.97),
                ]),
            }),
            Rc::new(TieredRollingRunbookBundleAdjuster {
                bounds: vec![344.0, 1373.0, 9823.0, 9902.0],
                rates: vec![0.87, 0.96, 0.81, 0.97],
            }),
            Rc::new(CapRollingRunbookBundleAdjuster { cap: 48052.0 }),
            Rc::new(RoundingRollingRunbookBundleAdjuster { step: 0.05 }),
            Rc::new(PercentRollingRunbookBundleAdjuster { percent: 1.1 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 43324.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_runbook_bundle_engine(&flags::new_registry());
        let item = new_rolling_runbook_bundle("code-946", "label-967");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
