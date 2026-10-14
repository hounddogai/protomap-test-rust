use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the silence target calculation.
pub trait SilenceTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSilenceTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SilenceTargetStep for SegmentSilenceTargetAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &SilenceTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceTargetAdjuster {
    pub percent: f64,
}

impl SilenceTargetStep for PercentSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSilenceTargetAdjuster {
    pub cap: f64,
}

impl SilenceTargetStep for CapSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SilenceTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSilenceTargetAdjuster {
    pub step: f64,
}

impl SilenceTargetStep for RoundingSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SilenceTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SilenceTargetStep for TieredSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SilenceTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTargetWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl SilenceTargetStep for SilenceTargetWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &SilenceTarget, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

/// Runs the silence target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceTargetEngine {
    pub steps: Vec<Rc<dyn SilenceTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceTargetEngine {
    pub fn compute(&mut self, item: &SilenceTarget) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SilenceTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_silence_target_engine() -> SilenceTargetEngine {
    SilenceTargetEngine {
        steps: vec![
            Rc::new(SegmentSilenceTargetAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.86)]),
            }) as Rc<dyn SilenceTargetStep>,
            Rc::new(PercentSilenceTargetAdjuster { percent: -4.1 }),
            Rc::new(CapSilenceTargetAdjuster { cap: 11530.0 }),
            Rc::new(RoundingSilenceTargetAdjuster { step: 0.25 }),
            Rc::new(TieredSilenceTargetAdjuster {
                bounds: vec![395.0, 5469.0, 7085.0],
                rates: vec![1.1, 0.85, 1.17],
            }),
            Rc::new(SilenceTargetWindowDaysOffsetAdjuster { weight: 1.78 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 16874.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_target_engine();
        let item = new_silence_target("code-894", "title-597");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
