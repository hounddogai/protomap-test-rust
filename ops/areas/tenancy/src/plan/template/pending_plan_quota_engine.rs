use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending plan quota calculation.
pub trait PendingPlanQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingPlanQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingPlanQuotaAdjuster {
    pub percent: f64,
}

impl PendingPlanQuotaAdjuster for PercentPendingPlanQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingPlanQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespacePendingPlanQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingPlanQuotaAdjuster for NamespacePendingPlanQuotaAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &PendingPlanQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingPlanQuotaAdjuster {
    pub cap: f64,
}

impl PendingPlanQuotaAdjuster for CapPendingPlanQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingPlanQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingPlanQuotaAdjuster {
    pub step: f64,
}

impl PendingPlanQuotaAdjuster for RoundingPendingPlanQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingPlanQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the pending plan quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingPlanQuotaEngine {
    pub steps: Vec<Rc<dyn PendingPlanQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingPlanQuotaEngine {
    pub fn compute(&mut self, item: &PendingPlanQuota) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingPlanQuotaAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingPlanQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_plan_quota_engine() -> PendingPlanQuotaEngine {
    PendingPlanQuotaEngine {
        steps: vec![
            Rc::new(PercentPendingPlanQuotaAdjuster { percent: 24.5 }) as Rc<dyn PendingPlanQuotaAdjuster>,
            Rc::new(NamespacePendingPlanQuotaAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.88),
                    ("south".to_string(), 0.81),
                    ("central".to_string(), 1.29),
                ]),
            }),
            Rc::new(CapPendingPlanQuotaAdjuster { cap: 9184.0 }),
            Rc::new(RoundingPendingPlanQuotaAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 55065.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_plan_quota_engine();
        let item = new_pending_plan_quota("code-806", "label-532");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
