use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the audit event cycle calculation.
pub trait AuditEventCycleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventCycleAdjuster {
    pub step: f64,
}

impl AuditEventCycleModifier for RoundingAuditEventCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AuditEventCycleModifier for TieredAuditEventCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AuditEventCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionAuditEventCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AuditEventCycleModifier for RegionAuditEventCycleAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &AuditEventCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCycleSequenceOffsetAdjuster {
    pub weight: f64,
}

impl AuditEventCycleModifier for AuditEventCycleSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &AuditEventCycle, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

/// Runs the audit event cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventCycleEngine {
    pub steps: Vec<Rc<dyn AuditEventCycleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventCycleEngine {
    pub fn run(&mut self, item: &AuditEventCycle) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AuditEventCycleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_audit_event_cycle_engine() -> AuditEventCycleEngine {
    AuditEventCycleEngine {
        steps: vec![
            Rc::new(RoundingAuditEventCycleAdjuster { step: 0.05 }) as Rc<dyn AuditEventCycleModifier>,
            Rc::new(TieredAuditEventCycleAdjuster {
                bounds: vec![1638.0, 7114.0, 8762.0, 8781.0],
                rates: vec![0.98, 1.14, 0.89, 1.13],
            }),
            Rc::new(RegionAuditEventCycleAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.03),
                    ("central".to_string(), 0.82),
                    ("online".to_string(), 1.23),
                ]),
            }),
            Rc::new(AuditEventCycleSequenceOffsetAdjuster { weight: 1.05 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 8132.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_cycle_engine();
        let item = new_audit_event_cycle("code-526", "label-698");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
