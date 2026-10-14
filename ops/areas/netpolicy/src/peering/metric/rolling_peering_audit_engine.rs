use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling peering audit calculation.
pub trait RollingPeeringAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPeeringAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingPeeringAuditAdjuster {
    pub step: f64,
}

impl RollingPeeringAuditFactor for RoundingRollingPeeringAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingPeeringAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingPeeringAuditAdjuster {
    pub cap: f64,
}

impl RollingPeeringAuditFactor for CapRollingPeeringAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingPeeringAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRollingPeeringAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingPeeringAuditFactor for ChannelRollingPeeringAuditAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RollingPeeringAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rolling peering audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPeeringAuditEngine {
    pub steps: Vec<Rc<dyn RollingPeeringAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPeeringAuditEngine {
    pub fn price(&mut self, item: &RollingPeeringAudit) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingPeeringAuditFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingPeeringAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_peering_audit_engine() -> RollingPeeringAuditEngine {
    RollingPeeringAuditEngine {
        steps: vec![
            Rc::new(RoundingRollingPeeringAuditAdjuster { step: 0.1 }) as Rc<dyn RollingPeeringAuditFactor>,
            Rc::new(CapRollingPeeringAuditAdjuster { cap: 10682.0 }),
            Rc::new(ChannelRollingPeeringAuditAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.19),
                    ("retail".to_string(), 1.35),
                    ("south".to_string(), 1.31),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 32124.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_peering_audit_engine();
        let item = new_rolling_peering_audit("code-148", "name-599");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
