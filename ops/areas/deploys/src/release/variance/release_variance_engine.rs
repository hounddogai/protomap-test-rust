use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the release variance calculation.
pub trait ReleaseVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseVarianceAdjuster {
    pub cap: f64,
}

impl ReleaseVarianceAdjuster for CapReleaseVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseVarianceAdjuster for FlaggedReleaseVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReleaseVarianceAdjuster {
    pub step: f64,
}

impl ReleaseVarianceAdjuster for RoundingReleaseVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReleaseVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReleaseVarianceAdjuster {
    pub percent: f64,
}

impl ReleaseVarianceAdjuster for PercentReleaseVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReleaseVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReleaseVarianceAdjuster {
    pub minimum: f64,
}

impl ReleaseVarianceAdjuster for MinimumReleaseVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReleaseVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the release variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseVarianceEngine {
    pub steps: Vec<Rc<dyn ReleaseVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseVarianceEngine {
    pub fn compute(&mut self, item: &ReleaseVariance) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseVarianceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_release_variance_engine(flags: &flags::Registry) -> ReleaseVarianceEngine {
    ReleaseVarianceEngine {
        steps: vec![
            Rc::new(CapReleaseVarianceAdjuster { cap: 26977.0 }) as Rc<dyn ReleaseVarianceAdjuster>,
            Rc::new(FlaggedReleaseVarianceAdjuster {
                flags: flags.clone(),
                flag: "release-variance-weighted".to_string(),
                percent: -7.1,
            }),
            Rc::new(RoundingReleaseVarianceAdjuster { step: 0.25 }),
            Rc::new(PercentReleaseVarianceAdjuster { percent: 11.0 }),
            Rc::new(MinimumReleaseVarianceAdjuster { minimum: 17.2 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 49018.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_variance_engine(&flags::new_registry());
        let item = new_release_variance("code-908", "label-155");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
