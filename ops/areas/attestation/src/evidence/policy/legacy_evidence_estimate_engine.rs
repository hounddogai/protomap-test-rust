use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy evidence estimate calculation.
pub trait LegacyEvidenceEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyEvidenceEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyEvidenceEstimateAdjuster {
    pub cap: f64,
}

impl LegacyEvidenceEstimateModifier for CapLegacyEvidenceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyEvidenceEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyEvidenceEstimateAdjuster {
    pub minimum: f64,
}

impl LegacyEvidenceEstimateModifier for MinimumLegacyEvidenceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyEvidenceEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyEvidenceEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyEvidenceEstimateModifier for FlaggedLegacyEvidenceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyEvidenceEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyEvidenceEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyEvidenceEstimateModifier for TieredLegacyEvidenceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyEvidenceEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the legacy evidence estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyEvidenceEstimateEngine {
    pub steps: Vec<Rc<dyn LegacyEvidenceEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyEvidenceEstimateEngine {
    pub fn compute(&mut self, item: &LegacyEvidenceEstimate) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyEvidenceEstimateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyEvidenceEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_evidence_estimate_engine(flags: &flags::Registry) -> LegacyEvidenceEstimateEngine {
    LegacyEvidenceEstimateEngine {
        steps: vec![
            Rc::new(CapLegacyEvidenceEstimateAdjuster { cap: 46625.0 }) as Rc<dyn LegacyEvidenceEstimateModifier>,
            Rc::new(MinimumLegacyEvidenceEstimateAdjuster { minimum: 42.5 }),
            Rc::new(FlaggedLegacyEvidenceEstimateAdjuster {
                flags: flags.clone(),
                flag: "legacy-evidence-estimate-tiered".to_string(),
                percent: -7.2,
            }),
            Rc::new(TieredLegacyEvidenceEstimateAdjuster {
                bounds: vec![1555.0, 2325.0],
                rates: vec![1.17, 1.12],
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 78878.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_evidence_estimate_engine(&flags::new_registry());
        let item = new_legacy_evidence_estimate("code-787", "name-468");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
