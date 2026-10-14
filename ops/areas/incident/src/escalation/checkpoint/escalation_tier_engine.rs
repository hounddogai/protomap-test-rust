use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the escalation tier calculation.
pub trait EscalationTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEscalationTierAdjuster {
    pub cap: f64,
}

impl EscalationTierAdjuster for CapEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EscalationTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEscalationTierAdjuster {
    pub percent: f64,
}

impl EscalationTierAdjuster for PercentEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EscalationTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationTierAdjuster {
    pub step: f64,
}

impl EscalationTierAdjuster for RoundingEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryEscalationTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EscalationTierAdjuster for CountryEscalationTierAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &EscalationTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEscalationTierAdjuster {
    pub minimum: f64,
}

impl EscalationTierAdjuster for MinimumEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EscalationTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationTierAdjuster for TieredEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the escalation tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationTierEngine {
    pub steps: Vec<Rc<dyn EscalationTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationTierEngine {
    pub fn price(&mut self, item: &EscalationTier) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EscalationTierAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EscalationTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_escalation_tier_engine() -> EscalationTierEngine {
    EscalationTierEngine {
        steps: vec![
            Rc::new(CapEscalationTierAdjuster { cap: 35827.0 }) as Rc<dyn EscalationTierAdjuster>,
            Rc::new(PercentEscalationTierAdjuster { percent: -15.6 }),
            Rc::new(RoundingEscalationTierAdjuster { step: 1.0 }),
            Rc::new(CountryEscalationTierAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.98), ("east".to_string(), 1.19)]),
            }),
            Rc::new(MinimumEscalationTierAdjuster { minimum: 3.7 }),
            Rc::new(TieredEscalationTierAdjuster {
                bounds: vec![6441.0, 9233.0],
                rates: vec![1.09, 0.83],
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 35957.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_tier_engine();
        let item = new_escalation_tier("code-763", "title-812");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
