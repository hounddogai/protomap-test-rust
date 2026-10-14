use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the peering audit calculation.
pub trait PeeringAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAuditMaximumOffsetAdjuster {
    pub weight: f64,
}

impl PeeringAuditAdjuster for PeeringAuditMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringAudit, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPeeringAuditAdjuster {
    pub cap: f64,
}

impl PeeringAuditAdjuster for CapPeeringAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PeeringAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PeeringAuditAdjuster for TieredPeeringAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PeeringAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPeeringAuditAdjuster {
    pub step: f64,
}

impl PeeringAuditAdjuster for RoundingPeeringAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PeeringAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPeeringAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PeeringAuditAdjuster for FlaggedPeeringAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PeeringAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the peering audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringAuditEngine {
    pub steps: Vec<Rc<dyn PeeringAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringAuditEngine {
    pub fn evaluate(&mut self, item: &PeeringAudit) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PeeringAuditAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PeeringAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_peering_audit_engine(flags: &flags::Registry) -> PeeringAuditEngine {
    PeeringAuditEngine {
        steps: vec![
            Rc::new(PeeringAuditMaximumOffsetAdjuster { weight: 0.15 }) as Rc<dyn PeeringAuditAdjuster>,
            Rc::new(CapPeeringAuditAdjuster { cap: 40559.0 }),
            Rc::new(TieredPeeringAuditAdjuster {
                bounds: vec![1121.0, 2412.0, 4883.0, 9891.0],
                rates: vec![0.96, 0.84, 0.81, 1.06],
            }),
            Rc::new(RoundingPeeringAuditAdjuster { step: 0.1 }),
            Rc::new(FlaggedPeeringAuditAdjuster {
                flags: flags.clone(),
                flag: "peering-audit-pending".to_string(),
                percent: 1.3,
            }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 54046.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_audit_engine(&flags::new_registry());
        let item = new_peering_audit("code-209", "title-321");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
