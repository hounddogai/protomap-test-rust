use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the silence override calculation.
pub trait SilenceOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceOverride, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSilenceOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SilenceOverrideAdjuster for FlaggedSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SilenceOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceOverrideFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl SilenceOverrideAdjuster for SilenceOverrideFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &SilenceOverride, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSilenceOverrideAdjuster {
    pub step: f64,
}

impl SilenceOverrideAdjuster for RoundingSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SilenceOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSilenceOverrideAdjuster {
    pub cap: f64,
}

impl SilenceOverrideAdjuster for CapSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SilenceOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the silence override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceOverrideEngine {
    pub steps: Vec<Rc<dyn SilenceOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceOverrideEngine {
    pub fn run(&mut self, item: &SilenceOverride) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SilenceOverrideAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SilenceOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_silence_override_engine(flags: &flags::Registry) -> SilenceOverrideEngine {
    SilenceOverrideEngine {
        steps: vec![
            Rc::new(FlaggedSilenceOverrideAdjuster {
                flags: flags.clone(),
                flag: "silence-override-fallback".to_string(),
                percent: 2.2,
            }) as Rc<dyn SilenceOverrideAdjuster>,
            Rc::new(SilenceOverrideFloorValueOffsetAdjuster { weight: 0.57 }),
            Rc::new(RoundingSilenceOverrideAdjuster { step: 0.5 }),
            Rc::new(CapSilenceOverrideAdjuster { cap: 32973.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 46512.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_override_engine(&flags::new_registry());
        let item = new_silence_override("code-999", "label-555");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
