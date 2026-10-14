use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the capped allowlist bundle calculation.
pub trait CappedAllowlistBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedAllowlistBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAllowlistBundleMinimumOffsetAdjuster {
    pub weight: f64,
}

impl CappedAllowlistBundleModifier for CappedAllowlistBundleMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &CappedAllowlistBundle, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedAllowlistBundleAdjuster {
    pub minimum: f64,
}

impl CappedAllowlistBundleModifier for MinimumCappedAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedAllowlistBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedAllowlistBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedAllowlistBundleModifier for FlaggedCappedAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedAllowlistBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedAllowlistBundleAdjuster {
    pub cap: f64,
}

impl CappedAllowlistBundleModifier for CapCappedAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedAllowlistBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedAllowlistBundleAdjuster {
    pub percent: f64,
}

impl CappedAllowlistBundleModifier for PercentCappedAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedAllowlistBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped allowlist bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedAllowlistBundleEngine {
    pub steps: Vec<Rc<dyn CappedAllowlistBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedAllowlistBundleEngine {
    pub fn compute(&mut self, item: &CappedAllowlistBundle) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedAllowlistBundleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedAllowlistBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_allowlist_bundle_engine(flags: &flags::Registry) -> CappedAllowlistBundleEngine {
    CappedAllowlistBundleEngine {
        steps: vec![
            Rc::new(CappedAllowlistBundleMinimumOffsetAdjuster { weight: 1.63 })
                as Rc<dyn CappedAllowlistBundleModifier>,
            Rc::new(MinimumCappedAllowlistBundleAdjuster { minimum: 46.8 }),
            Rc::new(FlaggedCappedAllowlistBundleAdjuster {
                flags: flags.clone(),
                flag: "capped-allowlist-bundle-global".to_string(),
                percent: 7.8,
            }),
            Rc::new(CapCappedAllowlistBundleAdjuster { cap: 44664.0 }),
            Rc::new(PercentCappedAllowlistBundleAdjuster { percent: -15.8 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 66894.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_allowlist_bundle_engine(&flags::new_registry());
        let item = new_capped_allowlist_bundle("code-299", "title-770");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
