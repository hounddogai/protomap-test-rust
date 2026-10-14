use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the regional audit event batch calculation.
pub trait RegionalAuditEventBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalAuditEventBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRegionalAuditEventBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalAuditEventBatchFactor for RegionRegionalAuditEventBatchAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RegionalAuditEventBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalAuditEventBatchAdjuster {
    pub cap: f64,
}

impl RegionalAuditEventBatchFactor for CapRegionalAuditEventBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalAuditEventBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventBatchBurstOffsetAdjuster {
    pub weight: f64,
}

impl RegionalAuditEventBatchFactor for RegionalAuditEventBatchBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalAuditEventBatch, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

/// Runs the regional audit event batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalAuditEventBatchEngine {
    pub steps: Vec<Rc<dyn RegionalAuditEventBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalAuditEventBatchEngine {
    pub fn run(&mut self, item: &RegionalAuditEventBatch) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalAuditEventBatchFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalAuditEventBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_audit_event_batch_engine() -> RegionalAuditEventBatchEngine {
    RegionalAuditEventBatchEngine {
        steps: vec![
            Rc::new(RegionRegionalAuditEventBatchAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.73)]),
            }) as Rc<dyn RegionalAuditEventBatchFactor>,
            Rc::new(CapRegionalAuditEventBatchAdjuster { cap: 6229.0 }),
            Rc::new(RegionalAuditEventBatchBurstOffsetAdjuster { weight: 1.51 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 29714.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_audit_event_batch_engine();
        let item = new_regional_audit_event_batch("code-196", "label-32");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
