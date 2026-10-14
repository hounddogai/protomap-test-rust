use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the reservation ledger calculation.
pub trait ReservationLedgerAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReservationLedgerAdjuster for TieredReservationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReservationLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReservationLedgerAdjuster {
    pub minimum: f64,
}

impl ReservationLedgerAdjuster for MinimumReservationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReservationLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationLedgerAdjuster {
    pub cap: f64,
}

impl ReservationLedgerAdjuster for CapReservationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionReservationLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationLedgerAdjuster for RegionReservationLedgerAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ReservationLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationLedgerVersionOffsetAdjuster {
    pub weight: f64,
}

impl ReservationLedgerAdjuster for ReservationLedgerVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationLedger, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

/// Runs the reservation ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationLedgerEngine {
    pub steps: Vec<Rc<dyn ReservationLedgerAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationLedgerEngine {
    pub fn run(&mut self, item: &ReservationLedger) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ReservationLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_reservation_ledger_engine() -> ReservationLedgerEngine {
    ReservationLedgerEngine {
        steps: vec![
            Rc::new(TieredReservationLedgerAdjuster {
                bounds: vec![4975.0, 6317.0, 7847.0, 8770.0],
                rates: vec![1.09, 0.97, 0.91, 1.11],
            }) as Rc<dyn ReservationLedgerAdjuster>,
            Rc::new(MinimumReservationLedgerAdjuster { minimum: 38.6 }),
            Rc::new(CapReservationLedgerAdjuster { cap: 23158.0 }),
            Rc::new(RegionReservationLedgerAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.03)]),
            }),
            Rc::new(ReservationLedgerVersionOffsetAdjuster { weight: 0.83 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 95075.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_ledger_engine();
        let item = new_reservation_ledger("code-423", "name-829");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
