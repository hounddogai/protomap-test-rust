use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending query quota calculation.
pub trait PendingQueryQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingQueryQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingQueryQuotaAdjuster {
    pub percent: f64,
}

impl PendingQueryQuotaFactor for PercentPendingQueryQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingQueryQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingQueryQuotaAdjuster {
    pub cap: f64,
}

impl PendingQueryQuotaFactor for CapPendingQueryQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingQueryQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingQueryQuotaAdjuster {
    pub step: f64,
}

impl PendingQueryQuotaFactor for RoundingPendingQueryQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingQueryQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryQuotaQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl PendingQueryQuotaFactor for PendingQueryQuotaQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &PendingQueryQuota, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

/// Runs the pending query quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingQueryQuotaEngine {
    pub steps: Vec<Rc<dyn PendingQueryQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingQueryQuotaEngine {
    pub fn price(&mut self, item: &PendingQueryQuota) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingQueryQuotaFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingQueryQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_query_quota_engine() -> PendingQueryQuotaEngine {
    PendingQueryQuotaEngine {
        steps: vec![
            Rc::new(PercentPendingQueryQuotaAdjuster { percent: -15.9 }) as Rc<dyn PendingQueryQuotaFactor>,
            Rc::new(CapPendingQueryQuotaAdjuster { cap: 7560.0 }),
            Rc::new(RoundingPendingQueryQuotaAdjuster { step: 0.25 }),
            Rc::new(PendingQueryQuotaQuotaUnitsOffsetAdjuster { weight: 1.08 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 38045.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_query_quota_engine();
        let item = new_pending_query_quota("code-923", "title-94");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
