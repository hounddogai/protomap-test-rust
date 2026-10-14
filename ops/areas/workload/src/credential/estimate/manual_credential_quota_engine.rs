use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual credential quota calculation.
pub trait ManualCredentialQuotaStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualCredentialQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialQuotaMinimumOffsetAdjuster {
    pub weight: f64,
}

impl ManualCredentialQuotaStep for ManualCredentialQuotaMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &ManualCredentialQuota, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualCredentialQuotaAdjuster {
    pub minimum: f64,
}

impl ManualCredentialQuotaStep for MinimumManualCredentialQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualCredentialQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualCredentialQuotaAdjuster {
    pub step: f64,
}

impl ManualCredentialQuotaStep for RoundingManualCredentialQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualCredentialQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual credential quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualCredentialQuotaEngine {
    pub steps: Vec<Rc<dyn ManualCredentialQuotaStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualCredentialQuotaEngine {
    pub fn compute(&mut self, item: &ManualCredentialQuota) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[ManualCredentialQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_credential_quota_engine() -> ManualCredentialQuotaEngine {
    ManualCredentialQuotaEngine {
        steps: vec![
            Rc::new(ManualCredentialQuotaMinimumOffsetAdjuster { weight: 1.11 }) as Rc<dyn ManualCredentialQuotaStep>,
            Rc::new(MinimumManualCredentialQuotaAdjuster { minimum: 43.0 }),
            Rc::new(RoundingManualCredentialQuotaAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 71042.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_credential_quota_engine();
        let item = new_manual_credential_quota("code-986", "name-177");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
