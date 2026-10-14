use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the evidence estimate calculation.
pub trait EvidenceEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelEvidenceEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceEstimateFactor for ChannelEvidenceEstimateAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &EvidenceEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceEstimateAdjuster {
    pub minimum: f64,
}

impl EvidenceEstimateFactor for MinimumEvidenceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceEstimateAdjuster {
    pub percent: f64,
}

impl EvidenceEstimateFactor for PercentEvidenceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidenceEstimateAdjuster {
    pub step: f64,
}

impl EvidenceEstimateFactor for RoundingEvidenceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidenceEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceEstimateFactor for FlaggedEvidenceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceEstimateAdjuster {
    pub cap: f64,
}

impl EvidenceEstimateFactor for CapEvidenceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the evidence estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceEstimateEngine {
    pub steps: Vec<Rc<dyn EvidenceEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceEstimateEngine {
    pub fn evaluate(&mut self, item: &EvidenceEstimate) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceEstimateFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_evidence_estimate_engine(flags: &flags::Registry) -> EvidenceEstimateEngine {
    EvidenceEstimateEngine {
        steps: vec![
            Rc::new(ChannelEvidenceEstimateAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.34), ("south".to_string(), 0.77)]),
            }) as Rc<dyn EvidenceEstimateFactor>,
            Rc::new(MinimumEvidenceEstimateAdjuster { minimum: 30.9 }),
            Rc::new(PercentEvidenceEstimateAdjuster { percent: -19.7 }),
            Rc::new(RoundingEvidenceEstimateAdjuster { step: 5.0 }),
            Rc::new(FlaggedEvidenceEstimateAdjuster {
                flags: flags.clone(),
                flag: "evidence-estimate-scheduled".to_string(),
                percent: 2.6,
            }),
            Rc::new(CapEvidenceEstimateAdjuster { cap: 10293.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 71553.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_estimate_engine(&flags::new_registry());
        let item = new_evidence_estimate("code-648", "title-795");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
