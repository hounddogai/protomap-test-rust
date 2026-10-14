use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the primary escalation notice calculation.
pub trait PrimaryEscalationNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryEscalationNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationNoticePortOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryEscalationNoticeAdjuster for PrimaryEscalationNoticePortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryEscalationNotice, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopePrimaryEscalationNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryEscalationNoticeAdjuster for ScopePrimaryEscalationNoticeAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &PrimaryEscalationNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryEscalationNoticeAdjuster {
    pub minimum: f64,
}

impl PrimaryEscalationNoticeAdjuster for MinimumPrimaryEscalationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryEscalationNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryEscalationNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryEscalationNoticeAdjuster for TieredPrimaryEscalationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryEscalationNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryEscalationNoticeAdjuster {
    pub cap: f64,
}

impl PrimaryEscalationNoticeAdjuster for CapPrimaryEscalationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryEscalationNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary escalation notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryEscalationNoticeEngine {
    pub steps: Vec<Rc<dyn PrimaryEscalationNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryEscalationNoticeEngine {
    pub fn price(&mut self, item: &PrimaryEscalationNotice) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryEscalationNoticeAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryEscalationNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_escalation_notice_engine() -> PrimaryEscalationNoticeEngine {
    PrimaryEscalationNoticeEngine {
        steps: vec![
            Rc::new(PrimaryEscalationNoticePortOffsetAdjuster { weight: 1.83 })
                as Rc<dyn PrimaryEscalationNoticeAdjuster>,
            Rc::new(ScopePrimaryEscalationNoticeAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.0), ("west".to_string(), 1.16)]),
            }),
            Rc::new(MinimumPrimaryEscalationNoticeAdjuster { minimum: 32.4 }),
            Rc::new(TieredPrimaryEscalationNoticeAdjuster {
                bounds: vec![1821.0, 2245.0, 4844.0, 5968.0],
                rates: vec![1.05, 0.85, 1.17, 1.2],
            }),
            Rc::new(CapPrimaryEscalationNoticeAdjuster { cap: 15336.0 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 37729.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_escalation_notice_engine();
        let item = new_primary_escalation_notice("code-351", "name-897");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
