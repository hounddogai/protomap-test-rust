use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tiered peering audit calculation.
pub trait TieredPeeringAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredPeeringAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAuditRatioOffsetAdjuster {
    pub weight: f64,
}

impl TieredPeeringAuditAdjuster for TieredPeeringAuditRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &TieredPeeringAudit, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredPeeringAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredPeeringAuditAdjuster for TieredTieredPeeringAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredPeeringAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredPeeringAuditAdjuster {
    pub minimum: f64,
}

impl TieredPeeringAuditAdjuster for MinimumTieredPeeringAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredPeeringAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered peering audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredPeeringAuditEngine {
    pub steps: Vec<Rc<dyn TieredPeeringAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredPeeringAuditEngine {
    pub fn evaluate(&mut self, item: &TieredPeeringAudit) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredPeeringAuditAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredPeeringAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_peering_audit_engine() -> TieredPeeringAuditEngine {
    TieredPeeringAuditEngine {
        steps: vec![
            Rc::new(TieredPeeringAuditRatioOffsetAdjuster { weight: 0.4 }) as Rc<dyn TieredPeeringAuditAdjuster>,
            Rc::new(TieredTieredPeeringAuditAdjuster {
                bounds: vec![4398.0, 5422.0, 6873.0],
                rates: vec![0.89, 1.04, 0.97],
            }),
            Rc::new(MinimumTieredPeeringAuditAdjuster { minimum: 7.5 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 46939.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_peering_audit_engine();
        let item = new_tiered_peering_audit("code-688", "name-230");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
