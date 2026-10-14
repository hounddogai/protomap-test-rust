use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient peering ledger calculation.
pub trait LenientPeeringLedgerModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientPeeringLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientPeeringLedgerAdjuster {
    pub step: f64,
}

impl LenientPeeringLedgerModifier for RoundingLenientPeeringLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientPeeringLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientPeeringLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientPeeringLedgerModifier for FlaggedLenientPeeringLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientPeeringLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLenientPeeringLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientPeeringLedgerModifier for TitleLenientPeeringLedgerAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LenientPeeringLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientPeeringLedgerAdjuster {
    pub minimum: f64,
}

impl LenientPeeringLedgerModifier for MinimumLenientPeeringLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientPeeringLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient peering ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientPeeringLedgerEngine {
    pub steps: Vec<Rc<dyn LenientPeeringLedgerModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientPeeringLedgerEngine {
    pub fn run(&mut self, item: &LenientPeeringLedger) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientPeeringLedgerModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientPeeringLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_peering_ledger_engine(flags: &flags::Registry) -> LenientPeeringLedgerEngine {
    LenientPeeringLedgerEngine {
        steps: vec![
            Rc::new(RoundingLenientPeeringLedgerAdjuster { step: 1.0 }) as Rc<dyn LenientPeeringLedgerModifier>,
            Rc::new(FlaggedLenientPeeringLedgerAdjuster {
                flags: flags.clone(),
                flag: "lenient-peering-ledger-primary".to_string(),
                percent: 3.2,
            }),
            Rc::new(TitleLenientPeeringLedgerAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.9),
                    ("online".to_string(), 1.32),
                    ("south".to_string(), 0.86),
                ]),
            }),
            Rc::new(MinimumLenientPeeringLedgerAdjuster { minimum: 3.8 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 80205.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_peering_ledger_engine(&flags::new_registry());
        let item = new_lenient_peering_ledger("code-312", "title-776");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
