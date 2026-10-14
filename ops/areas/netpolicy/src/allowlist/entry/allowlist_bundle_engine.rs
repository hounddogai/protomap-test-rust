use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the allowlist bundle calculation.
pub trait AllowlistBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistBundle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistBundleFactor for FlaggedAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAllowlistBundleAdjuster {
    pub minimum: f64,
}

impl AllowlistBundleFactor for MinimumAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AllowlistBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelAllowlistBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AllowlistBundleFactor for LabelAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &AllowlistBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AllowlistBundleFactor for TieredAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AllowlistBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAllowlistBundleAdjuster {
    pub step: f64,
}

impl AllowlistBundleFactor for RoundingAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AllowlistBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistBundleAdjuster {
    pub cap: f64,
}

impl AllowlistBundleFactor for CapAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the allowlist bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistBundleEngine {
    pub steps: Vec<Rc<dyn AllowlistBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistBundleEngine {
    pub fn run(&mut self, item: &AllowlistBundle) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_allowlist_bundle_engine(flags: &flags::Registry) -> AllowlistBundleEngine {
    AllowlistBundleEngine {
        steps: vec![
            Rc::new(FlaggedAllowlistBundleAdjuster {
                flags: flags.clone(),
                flag: "allowlist-bundle-scheduled".to_string(),
                percent: 7.7,
            }) as Rc<dyn AllowlistBundleFactor>,
            Rc::new(MinimumAllowlistBundleAdjuster { minimum: 17.8 }),
            Rc::new(LabelAllowlistBundleAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.15),
                    ("retail".to_string(), 0.78),
                    ("west".to_string(), 0.75),
                ]),
            }),
            Rc::new(TieredAllowlistBundleAdjuster {
                bounds: vec![654.0, 8157.0, 9424.0],
                rates: vec![0.83, 1.13, 0.9],
            }),
            Rc::new(RoundingAllowlistBundleAdjuster { step: 0.1 }),
            Rc::new(CapAllowlistBundleAdjuster { cap: 20611.0 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 29042.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_allowlist_bundle_engine(&flags::new_registry());
        let item = new_allowlist_bundle("code-593", "label-671");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
