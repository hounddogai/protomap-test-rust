use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual silence variance calculation.
pub trait ManualSilenceVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualSilenceVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceVarianceUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl ManualSilenceVarianceFactor for ManualSilenceVarianceUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &ManualSilenceVariance, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualSilenceVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualSilenceVarianceFactor for FlaggedManualSilenceVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualSilenceVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualSilenceVarianceAdjuster {
    pub cap: f64,
}

impl ManualSilenceVarianceFactor for CapManualSilenceVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualSilenceVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the manual silence variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualSilenceVarianceEngine {
    pub steps: Vec<Rc<dyn ManualSilenceVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualSilenceVarianceEngine {
    pub fn compute(&mut self, item: &ManualSilenceVariance) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualSilenceVarianceFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualSilenceVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_silence_variance_engine(flags: &flags::Registry) -> ManualSilenceVarianceEngine {
    ManualSilenceVarianceEngine {
        steps: vec![
            Rc::new(ManualSilenceVarianceUpdatedAtOffsetAdjuster { weight: 1.93 })
                as Rc<dyn ManualSilenceVarianceFactor>,
            Rc::new(FlaggedManualSilenceVarianceAdjuster {
                flags: flags.clone(),
                flag: "manual-silence-variance-seasonal".to_string(),
                percent: 0.3,
            }),
            Rc::new(CapManualSilenceVarianceAdjuster { cap: 20482.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 41873.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_silence_variance_engine(&flags::new_registry());
        let item = new_manual_silence_variance("code-966", "label-907");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
