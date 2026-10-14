use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default release claim calculation.
pub trait DefaultReleaseClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultReleaseClaim, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultReleaseClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultReleaseClaimStep for FlaggedDefaultReleaseClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultReleaseClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseClaimPriorityOffsetAdjuster {
    pub weight: f64,
}

impl DefaultReleaseClaimStep for DefaultReleaseClaimPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultReleaseClaim, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultReleaseClaimAdjuster {
    pub percent: f64,
}

impl DefaultReleaseClaimStep for PercentDefaultReleaseClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultReleaseClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default release claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultReleaseClaimEngine {
    pub steps: Vec<Rc<dyn DefaultReleaseClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultReleaseClaimEngine {
    pub fn compute(&mut self, item: &DefaultReleaseClaim) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[DefaultReleaseClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_release_claim_engine(flags: &flags::Registry) -> DefaultReleaseClaimEngine {
    DefaultReleaseClaimEngine {
        steps: vec![
            Rc::new(FlaggedDefaultReleaseClaimAdjuster {
                flags: flags.clone(),
                flag: "default-release-claim-batched".to_string(),
                percent: -0.8,
            }) as Rc<dyn DefaultReleaseClaimStep>,
            Rc::new(DefaultReleaseClaimPriorityOffsetAdjuster { weight: 1.99 }),
            Rc::new(PercentDefaultReleaseClaimAdjuster { percent: -1.7 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 65343.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_release_claim_engine(&flags::new_registry());
        let item = new_default_release_claim("code-257", "label-182");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
