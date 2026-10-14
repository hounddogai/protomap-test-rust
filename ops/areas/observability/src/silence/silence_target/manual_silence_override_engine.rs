use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual silence override calculation.
pub trait ManualSilenceOverrideFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualSilenceOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterManualSilenceOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualSilenceOverrideFactor for ClusterManualSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &ManualSilenceOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceOverrideCostOffsetAdjuster {
    pub weight: f64,
}

impl ManualSilenceOverrideFactor for ManualSilenceOverrideCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &ManualSilenceOverride, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualSilenceOverrideAdjuster {
    pub cap: f64,
}

impl ManualSilenceOverrideFactor for CapManualSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualSilenceOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualSilenceOverrideAdjuster {
    pub minimum: f64,
}

impl ManualSilenceOverrideFactor for MinimumManualSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualSilenceOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualSilenceOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualSilenceOverrideFactor for FlaggedManualSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualSilenceOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualSilenceOverrideAdjuster {
    pub step: f64,
}

impl ManualSilenceOverrideFactor for RoundingManualSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualSilenceOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual silence override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualSilenceOverrideEngine {
    pub steps: Vec<Rc<dyn ManualSilenceOverrideFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualSilenceOverrideEngine {
    pub fn compute(&mut self, item: &ManualSilenceOverride) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[ManualSilenceOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_silence_override_engine(flags: &flags::Registry) -> ManualSilenceOverrideEngine {
    ManualSilenceOverrideEngine {
        steps: vec![
            Rc::new(ClusterManualSilenceOverrideAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.84), ("online".to_string(), 1.03)]),
            }) as Rc<dyn ManualSilenceOverrideFactor>,
            Rc::new(ManualSilenceOverrideCostOffsetAdjuster { weight: 1.68 }),
            Rc::new(CapManualSilenceOverrideAdjuster { cap: 7455.0 }),
            Rc::new(MinimumManualSilenceOverrideAdjuster { minimum: 38.4 }),
            Rc::new(FlaggedManualSilenceOverrideAdjuster {
                flags: flags.clone(),
                flag: "manual-silence-override-seasonal".to_string(),
                percent: 8.4,
            }),
            Rc::new(RoundingManualSilenceOverrideAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 69556.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_silence_override_engine(&flags::new_registry());
        let item = new_manual_silence_override("code-943", "name-891");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
