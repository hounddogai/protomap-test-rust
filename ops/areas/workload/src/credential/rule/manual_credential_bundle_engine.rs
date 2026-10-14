use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual credential bundle calculation.
pub trait ManualCredentialBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualCredentialBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualCredentialBundleAdjuster {
    pub cap: f64,
}

impl ManualCredentialBundleFactor for CapManualCredentialBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualCredentialBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualCredentialBundleAdjuster {
    pub percent: f64,
}

impl ManualCredentialBundleFactor for PercentManualCredentialBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualCredentialBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualCredentialBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualCredentialBundleFactor for FlaggedManualCredentialBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualCredentialBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialBundleDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl ManualCredentialBundleFactor for ManualCredentialBundleDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &ManualCredentialBundle, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualCredentialBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualCredentialBundleFactor for TieredManualCredentialBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualCredentialBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual credential bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualCredentialBundleEngine {
    pub steps: Vec<Rc<dyn ManualCredentialBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualCredentialBundleEngine {
    pub fn evaluate(&mut self, item: &ManualCredentialBundle) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[ManualCredentialBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_credential_bundle_engine(flags: &flags::Registry) -> ManualCredentialBundleEngine {
    ManualCredentialBundleEngine {
        steps: vec![
            Rc::new(CapManualCredentialBundleAdjuster { cap: 49027.0 }) as Rc<dyn ManualCredentialBundleFactor>,
            Rc::new(PercentManualCredentialBundleAdjuster { percent: -24.5 }),
            Rc::new(FlaggedManualCredentialBundleAdjuster {
                flags: flags.clone(),
                flag: "manual-credential-bundle-default".to_string(),
                percent: 8.3,
            }),
            Rc::new(ManualCredentialBundleDurationMinutesOffsetAdjuster { weight: 1.56 }),
            Rc::new(TieredManualCredentialBundleAdjuster {
                bounds: vec![3553.0, 7640.0],
                rates: vec![0.83, 1.12],
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 7563.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_credential_bundle_engine(&flags::new_registry());
        let item = new_manual_credential_bundle("code-380", "title-47");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
