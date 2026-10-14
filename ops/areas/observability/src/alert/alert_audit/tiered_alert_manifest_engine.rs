use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tiered alert manifest calculation.
pub trait TieredAlertManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredAlertManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredAlertManifestAdjuster {
    pub percent: f64,
}

impl TieredAlertManifestStep for PercentTieredAlertManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredAlertManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredAlertManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredAlertManifestStep for TieredTieredAlertManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredAlertManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredAlertManifestAdjuster {
    pub minimum: f64,
}

impl TieredAlertManifestStep for MinimumTieredAlertManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredAlertManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredAlertManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredAlertManifestStep for FlaggedTieredAlertManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredAlertManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredAlertManifestAdjuster {
    pub cap: f64,
}

impl TieredAlertManifestStep for CapTieredAlertManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredAlertManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeTieredAlertManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredAlertManifestStep for ScopeTieredAlertManifestAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &TieredAlertManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the tiered alert manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredAlertManifestEngine {
    pub steps: Vec<Rc<dyn TieredAlertManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredAlertManifestEngine {
    pub fn compute(&mut self, item: &TieredAlertManifest) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredAlertManifestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredAlertManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_alert_manifest_engine(flags: &flags::Registry) -> TieredAlertManifestEngine {
    TieredAlertManifestEngine {
        steps: vec![
            Rc::new(PercentTieredAlertManifestAdjuster { percent: -12.4 }) as Rc<dyn TieredAlertManifestStep>,
            Rc::new(TieredTieredAlertManifestAdjuster {
                bounds: vec![1705.0, 7326.0, 7746.0],
                rates: vec![1.13, 1.07, 0.93],
            }),
            Rc::new(MinimumTieredAlertManifestAdjuster { minimum: 30.7 }),
            Rc::new(FlaggedTieredAlertManifestAdjuster {
                flags: flags.clone(),
                flag: "tiered-alert-manifest-weighted".to_string(),
                percent: -1.9,
            }),
            Rc::new(CapTieredAlertManifestAdjuster { cap: 11747.0 }),
            Rc::new(ScopeTieredAlertManifestAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.31)]),
            }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 58579.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_alert_manifest_engine(&flags::new_registry());
        let item = new_tiered_alert_manifest("code-618", "label-143");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
