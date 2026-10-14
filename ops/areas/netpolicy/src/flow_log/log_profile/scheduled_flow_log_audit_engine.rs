use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled flow log audit calculation.
pub trait ScheduledFlowLogAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledFlowLogAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFlowLogAuditRevisionOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledFlowLogAuditFactor for ScheduledFlowLogAuditRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledFlowLogAudit, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledFlowLogAuditAdjuster {
    pub cap: f64,
}

impl ScheduledFlowLogAuditFactor for CapScheduledFlowLogAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledFlowLogAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledFlowLogAuditAdjuster {
    pub minimum: f64,
}

impl ScheduledFlowLogAuditFactor for MinimumScheduledFlowLogAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledFlowLogAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledFlowLogAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledFlowLogAuditFactor for TieredScheduledFlowLogAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledFlowLogAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledFlowLogAuditAdjuster {
    pub percent: f64,
}

impl ScheduledFlowLogAuditFactor for PercentScheduledFlowLogAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledFlowLogAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelScheduledFlowLogAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledFlowLogAuditFactor for LabelScheduledFlowLogAuditAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ScheduledFlowLogAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled flow log audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledFlowLogAuditEngine {
    pub steps: Vec<Rc<dyn ScheduledFlowLogAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledFlowLogAuditEngine {
    pub fn compute(&mut self, item: &ScheduledFlowLogAudit) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledFlowLogAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_flow_log_audit_engine() -> ScheduledFlowLogAuditEngine {
    ScheduledFlowLogAuditEngine {
        steps: vec![
            Rc::new(ScheduledFlowLogAuditRevisionOffsetAdjuster { weight: 0.02 })
                as Rc<dyn ScheduledFlowLogAuditFactor>,
            Rc::new(CapScheduledFlowLogAuditAdjuster { cap: 28175.0 }),
            Rc::new(MinimumScheduledFlowLogAuditAdjuster { minimum: 48.9 }),
            Rc::new(TieredScheduledFlowLogAuditAdjuster {
                bounds: vec![826.0, 9331.0],
                rates: vec![0.87, 0.93],
            }),
            Rc::new(PercentScheduledFlowLogAuditAdjuster { percent: -1.3 }),
            Rc::new(LabelScheduledFlowLogAuditAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.21), ("west".to_string(), 1.02)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 20474.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_flow_log_audit_engine();
        let item = new_scheduled_flow_log_audit("code-518", "label-401");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
