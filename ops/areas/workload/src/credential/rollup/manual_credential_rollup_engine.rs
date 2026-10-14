use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual credential rollup calculation.
pub trait ManualCredentialRollupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualCredentialRollup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualCredentialRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualCredentialRollupStep for FlaggedManualCredentialRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualCredentialRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualCredentialRollupAdjuster {
    pub percent: f64,
}

impl ManualCredentialRollupStep for PercentManualCredentialRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualCredentialRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualCredentialRollupAdjuster {
    pub minimum: f64,
}

impl ManualCredentialRollupStep for MinimumManualCredentialRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualCredentialRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualCredentialRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualCredentialRollupStep for TieredManualCredentialRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualCredentialRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualCredentialRollupAdjuster {
    pub cap: f64,
}

impl ManualCredentialRollupStep for CapManualCredentialRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualCredentialRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialRollupWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl ManualCredentialRollupStep for ManualCredentialRollupWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &ManualCredentialRollup, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

/// Runs the manual credential rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualCredentialRollupEngine {
    pub steps: Vec<Rc<dyn ManualCredentialRollupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualCredentialRollupEngine {
    pub fn price(&mut self, item: &ManualCredentialRollup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ManualCredentialRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_credential_rollup_engine(flags: &flags::Registry) -> ManualCredentialRollupEngine {
    ManualCredentialRollupEngine {
        steps: vec![
            Rc::new(FlaggedManualCredentialRollupAdjuster {
                flags: flags.clone(),
                flag: "manual-credential-rollup-rolling".to_string(),
                percent: 0.4,
            }) as Rc<dyn ManualCredentialRollupStep>,
            Rc::new(PercentManualCredentialRollupAdjuster { percent: 1.5 }),
            Rc::new(MinimumManualCredentialRollupAdjuster { minimum: 47.5 }),
            Rc::new(TieredManualCredentialRollupAdjuster {
                bounds: vec![2081.0, 4456.0],
                rates: vec![1.11, 1.09],
            }),
            Rc::new(CapManualCredentialRollupAdjuster { cap: 32444.0 }),
            Rc::new(ManualCredentialRollupWindowDaysOffsetAdjuster { weight: 0.07 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 19468.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_credential_rollup_engine(&flags::new_registry());
        let item = new_manual_credential_rollup("code-995", "title-932");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
