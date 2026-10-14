use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched data export reservation calculation.
pub trait BatchedDataExportReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedDataExportReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedDataExportReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedDataExportReservationStep for TieredBatchedDataExportReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedDataExportReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDataExportReservationLimitOffsetAdjuster {
    pub weight: f64,
}

impl BatchedDataExportReservationStep for BatchedDataExportReservationLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedDataExportReservation, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedDataExportReservationAdjuster {
    pub cap: f64,
}

impl BatchedDataExportReservationStep for CapBatchedDataExportReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedDataExportReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedDataExportReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedDataExportReservationStep for FlaggedBatchedDataExportReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedDataExportReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedDataExportReservationAdjuster {
    pub percent: f64,
}

impl BatchedDataExportReservationStep for PercentBatchedDataExportReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedDataExportReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched data export reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedDataExportReservationEngine {
    pub steps: Vec<Rc<dyn BatchedDataExportReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedDataExportReservationEngine {
    pub fn run(&mut self, item: &BatchedDataExportReservation) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedDataExportReservationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedDataExportReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_data_export_reservation_engine(flags: &flags::Registry) -> BatchedDataExportReservationEngine {
    BatchedDataExportReservationEngine {
        steps: vec![
            Rc::new(TieredBatchedDataExportReservationAdjuster {
                bounds: vec![5612.0, 7455.0],
                rates: vec![1.17, 1.04],
            }) as Rc<dyn BatchedDataExportReservationStep>,
            Rc::new(BatchedDataExportReservationLimitOffsetAdjuster { weight: 1.08 }),
            Rc::new(CapBatchedDataExportReservationAdjuster { cap: 15589.0 }),
            Rc::new(FlaggedBatchedDataExportReservationAdjuster {
                flags: flags.clone(),
                flag: "batched-data-export-reservation-primary".to_string(),
                percent: -3.3,
            }),
            Rc::new(PercentBatchedDataExportReservationAdjuster { percent: 1.9 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 81952.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_data_export_reservation_engine(&flags::new_registry());
        let item = new_batched_data_export_reservation("code-8", "title-397");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
