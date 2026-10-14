use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy verifier estimate calculation.
pub trait LegacyVerifierEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyVerifierEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyVerifierEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyVerifierEstimateStep for FlaggedLegacyVerifierEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyVerifierEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyVerifierEstimateAdjuster {
    pub percent: f64,
}

impl LegacyVerifierEstimateStep for PercentLegacyVerifierEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyVerifierEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyVerifierEstimateAdjuster {
    pub step: f64,
}

impl LegacyVerifierEstimateStep for RoundingLegacyVerifierEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyVerifierEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyVerifierEstimateAdjuster {
    pub minimum: f64,
}

impl LegacyVerifierEstimateStep for MinimumLegacyVerifierEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyVerifierEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy verifier estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyVerifierEstimateEngine {
    pub steps: Vec<Rc<dyn LegacyVerifierEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyVerifierEstimateEngine {
    pub fn evaluate(&mut self, item: &LegacyVerifierEstimate) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyVerifierEstimateStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyVerifierEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_verifier_estimate_engine(flags: &flags::Registry) -> LegacyVerifierEstimateEngine {
    LegacyVerifierEstimateEngine {
        steps: vec![
            Rc::new(FlaggedLegacyVerifierEstimateAdjuster {
                flags: flags.clone(),
                flag: "legacy-verifier-estimate-scheduled".to_string(),
                percent: 7.4,
            }) as Rc<dyn LegacyVerifierEstimateStep>,
            Rc::new(PercentLegacyVerifierEstimateAdjuster { percent: -22.9 }),
            Rc::new(RoundingLegacyVerifierEstimateAdjuster { step: 0.5 }),
            Rc::new(MinimumLegacyVerifierEstimateAdjuster { minimum: 5.5 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 39273.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_verifier_estimate_engine(&flags::new_registry());
        let item = new_legacy_verifier_estimate("code-97", "label-200");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
