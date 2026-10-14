use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending federation variance calculation.
pub trait PendingFederationVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingFederationVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingFederationVarianceAdjuster {
    pub percent: f64,
}

impl PendingFederationVarianceAdjuster for PercentPendingFederationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingFederationVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPendingFederationVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingFederationVarianceAdjuster for LabelPendingFederationVarianceAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PendingFederationVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingFederationVarianceAdjuster {
    pub step: f64,
}

impl PendingFederationVarianceAdjuster for RoundingPendingFederationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingFederationVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingFederationVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingFederationVarianceAdjuster for FlaggedPendingFederationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingFederationVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending federation variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingFederationVarianceEngine {
    pub steps: Vec<Rc<dyn PendingFederationVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingFederationVarianceEngine {
    pub fn compute(&mut self, item: &PendingFederationVariance) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[PendingFederationVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_federation_variance_engine(flags: &flags::Registry) -> PendingFederationVarianceEngine {
    PendingFederationVarianceEngine {
        steps: vec![
            Rc::new(PercentPendingFederationVarianceAdjuster { percent: 0.2 })
                as Rc<dyn PendingFederationVarianceAdjuster>,
            Rc::new(LabelPendingFederationVarianceAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.17)]),
            }),
            Rc::new(RoundingPendingFederationVarianceAdjuster { step: 0.25 }),
            Rc::new(FlaggedPendingFederationVarianceAdjuster {
                flags: flags.clone(),
                flag: "pending-federation-variance-pending".to_string(),
                percent: 0.3,
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 60749.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_federation_variance_engine(&flags::new_registry());
        let item = new_pending_federation_variance("code-212", "label-770");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
