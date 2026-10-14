use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the escalation digest calculation.
pub trait EscalationDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationDigestAdjuster for TieredEscalationDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEscalationDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EscalationDigestAdjuster for FlaggedEscalationDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EscalationDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationDigestAdjuster {
    pub step: f64,
}

impl EscalationDigestAdjuster for RoundingEscalationDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEscalationDigestAdjuster {
    pub percent: f64,
}

impl EscalationDigestAdjuster for PercentEscalationDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EscalationDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the escalation digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationDigestEngine {
    pub steps: Vec<Rc<dyn EscalationDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationDigestEngine {
    pub fn price(&mut self, item: &EscalationDigest) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[EscalationDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_escalation_digest_engine(flags: &flags::Registry) -> EscalationDigestEngine {
    EscalationDigestEngine {
        steps: vec![
            Rc::new(TieredEscalationDigestAdjuster {
                bounds: vec![2182.0, 6197.0],
                rates: vec![0.81, 1.17],
            }) as Rc<dyn EscalationDigestAdjuster>,
            Rc::new(FlaggedEscalationDigestAdjuster {
                flags: flags.clone(),
                flag: "escalation-digest-default".to_string(),
                percent: 0.0,
            }),
            Rc::new(RoundingEscalationDigestAdjuster { step: 0.5 }),
            Rc::new(PercentEscalationDigestAdjuster { percent: 2.4 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 46236.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_digest_engine(&flags::new_registry());
        let item = new_escalation_digest("code-17", "title-922");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
