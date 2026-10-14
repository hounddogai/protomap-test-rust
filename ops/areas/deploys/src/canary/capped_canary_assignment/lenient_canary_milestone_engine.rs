use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient canary milestone calculation.
pub trait LenientCanaryMilestoneAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientCanaryMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientCanaryMilestoneAdjuster {
    pub cap: f64,
}

impl LenientCanaryMilestoneAdjuster for CapLenientCanaryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientCanaryMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientCanaryMilestoneAdjuster {
    pub minimum: f64,
}

impl LenientCanaryMilestoneAdjuster for MinimumLenientCanaryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientCanaryMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientCanaryMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientCanaryMilestoneAdjuster for TieredLenientCanaryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientCanaryMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientCanaryMilestoneAdjuster {
    pub percent: f64,
}

impl LenientCanaryMilestoneAdjuster for PercentLenientCanaryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientCanaryMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientCanaryMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientCanaryMilestoneAdjuster for FlaggedLenientCanaryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientCanaryMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient canary milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientCanaryMilestoneEngine {
    pub steps: Vec<Rc<dyn LenientCanaryMilestoneAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientCanaryMilestoneEngine {
    pub fn run(&mut self, item: &LenientCanaryMilestone) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientCanaryMilestoneAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientCanaryMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_canary_milestone_engine(flags: &flags::Registry) -> LenientCanaryMilestoneEngine {
    LenientCanaryMilestoneEngine {
        steps: vec![
            Rc::new(CapLenientCanaryMilestoneAdjuster { cap: 43625.0 }) as Rc<dyn LenientCanaryMilestoneAdjuster>,
            Rc::new(MinimumLenientCanaryMilestoneAdjuster { minimum: 11.3 }),
            Rc::new(TieredLenientCanaryMilestoneAdjuster {
                bounds: vec![9695.0, 9923.0],
                rates: vec![1.17, 0.81],
            }),
            Rc::new(PercentLenientCanaryMilestoneAdjuster { percent: 1.2 }),
            Rc::new(FlaggedLenientCanaryMilestoneAdjuster {
                flags: flags.clone(),
                flag: "lenient-canary-milestone-fallback".to_string(),
                percent: -9.0,
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 53696.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_canary_milestone_engine(&flags::new_registry());
        let item = new_lenient_canary_milestone("code-917", "label-584");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
