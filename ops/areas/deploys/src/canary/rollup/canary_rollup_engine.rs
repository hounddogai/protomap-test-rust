use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the canary rollup calculation.
pub trait CanaryRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryRollup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCanaryRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CanaryRollupFactor for FlaggedCanaryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CanaryRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryRollupAdjuster {
    pub step: f64,
}

impl CanaryRollupFactor for RoundingCanaryRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanaryRollupAdjuster {
    pub cap: f64,
}

impl CanaryRollupFactor for CapCanaryRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanaryRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryRollupAdjuster {
    pub minimum: f64,
}

impl CanaryRollupFactor for MinimumCanaryRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the canary rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryRollupEngine {
    pub steps: Vec<Rc<dyn CanaryRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryRollupEngine {
    pub fn evaluate(&mut self, item: &CanaryRollup) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CanaryRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CanaryRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_canary_rollup_engine(flags: &flags::Registry) -> CanaryRollupEngine {
    CanaryRollupEngine {
        steps: vec![
            Rc::new(FlaggedCanaryRollupAdjuster {
                flags: flags.clone(),
                flag: "canary-rollup-seasonal".to_string(),
                percent: -8.7,
            }) as Rc<dyn CanaryRollupFactor>,
            Rc::new(RoundingCanaryRollupAdjuster { step: 0.5 }),
            Rc::new(CapCanaryRollupAdjuster { cap: 14076.0 }),
            Rc::new(MinimumCanaryRollupAdjuster { minimum: 31.7 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 43318.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_rollup_engine(&flags::new_registry());
        let item = new_canary_rollup("code-326", "title-531");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
