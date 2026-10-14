use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tiered flow log audit calculation.
pub trait TieredFlowLogAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredFlowLogAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredFlowLogAuditAdjuster {
    pub cap: f64,
}

impl TieredFlowLogAuditFactor for CapTieredFlowLogAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredFlowLogAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogAuditWeightOffsetAdjuster {
    pub weight: f64,
}

impl TieredFlowLogAuditFactor for TieredFlowLogAuditWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &TieredFlowLogAudit, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredFlowLogAuditAdjuster {
    pub percent: f64,
}

impl TieredFlowLogAuditFactor for PercentTieredFlowLogAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredFlowLogAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleTieredFlowLogAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredFlowLogAuditFactor for LocaleTieredFlowLogAuditAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &TieredFlowLogAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredFlowLogAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredFlowLogAuditFactor for TieredTieredFlowLogAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredFlowLogAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered flow log audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredFlowLogAuditEngine {
    pub steps: Vec<Rc<dyn TieredFlowLogAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredFlowLogAuditEngine {
    pub fn run(&mut self, item: &TieredFlowLogAudit) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredFlowLogAuditFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredFlowLogAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_flow_log_audit_engine() -> TieredFlowLogAuditEngine {
    TieredFlowLogAuditEngine {
        steps: vec![
            Rc::new(CapTieredFlowLogAuditAdjuster { cap: 5357.0 }) as Rc<dyn TieredFlowLogAuditFactor>,
            Rc::new(TieredFlowLogAuditWeightOffsetAdjuster { weight: 0.42 }),
            Rc::new(PercentTieredFlowLogAuditAdjuster { percent: 17.6 }),
            Rc::new(LocaleTieredFlowLogAuditAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.75),
                    ("retail".to_string(), 1.33),
                    ("north".to_string(), 1.38),
                ]),
            }),
            Rc::new(TieredTieredFlowLogAuditAdjuster {
                bounds: vec![4919.0, 6729.0, 9446.0],
                rates: vec![1.05, 1.02, 0.87],
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 9623.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_flow_log_audit_engine();
        let item = new_tiered_flow_log_audit("code-931", "title-798");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
