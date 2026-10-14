use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending ttl policy group calculation.
pub trait PendingTtlPolicyGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingTtlPolicyGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyGroupBurstOffsetAdjuster {
    pub weight: f64,
}

impl PendingTtlPolicyGroupAdjuster for PendingTtlPolicyGroupBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &PendingTtlPolicyGroup, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespacePendingTtlPolicyGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingTtlPolicyGroupAdjuster for NamespacePendingTtlPolicyGroupAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &PendingTtlPolicyGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingTtlPolicyGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingTtlPolicyGroupAdjuster for FlaggedPendingTtlPolicyGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingTtlPolicyGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingTtlPolicyGroupAdjuster {
    pub cap: f64,
}

impl PendingTtlPolicyGroupAdjuster for CapPendingTtlPolicyGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingTtlPolicyGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the pending ttl policy group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingTtlPolicyGroupEngine {
    pub steps: Vec<Rc<dyn PendingTtlPolicyGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingTtlPolicyGroupEngine {
    pub fn compute(&mut self, item: &PendingTtlPolicyGroup) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[PendingTtlPolicyGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_ttl_policy_group_engine(flags: &flags::Registry) -> PendingTtlPolicyGroupEngine {
    PendingTtlPolicyGroupEngine {
        steps: vec![
            Rc::new(PendingTtlPolicyGroupBurstOffsetAdjuster { weight: 1.33 }) as Rc<dyn PendingTtlPolicyGroupAdjuster>,
            Rc::new(NamespacePendingTtlPolicyGroupAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.15),
                    ("online".to_string(), 0.71),
                    ("north".to_string(), 0.79),
                ]),
            }),
            Rc::new(FlaggedPendingTtlPolicyGroupAdjuster {
                flags: flags.clone(),
                flag: "pending-ttl-policy-group-capped".to_string(),
                percent: 3.3,
            }),
            Rc::new(CapPendingTtlPolicyGroupAdjuster { cap: 29902.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 60409.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_ttl_policy_group_engine(&flags::new_registry());
        let item = new_pending_ttl_policy_group("code-501", "title-855");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
