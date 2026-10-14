use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual dns record reservation calculation.
pub trait ManualDnsRecordReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualDnsRecordReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualDnsRecordReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualDnsRecordReservationStep for TieredManualDnsRecordReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualDnsRecordReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualDnsRecordReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualDnsRecordReservationStep for FlaggedManualDnsRecordReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualDnsRecordReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualDnsRecordReservationAdjuster {
    pub percent: f64,
}

impl ManualDnsRecordReservationStep for PercentManualDnsRecordReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualDnsRecordReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualDnsRecordReservationAdjuster {
    pub minimum: f64,
}

impl ManualDnsRecordReservationStep for MinimumManualDnsRecordReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualDnsRecordReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordReservationThresholdOffsetAdjuster {
    pub weight: f64,
}

impl ManualDnsRecordReservationStep for ManualDnsRecordReservationThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &ManualDnsRecordReservation, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

/// Runs the manual dns record reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualDnsRecordReservationEngine {
    pub steps: Vec<Rc<dyn ManualDnsRecordReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualDnsRecordReservationEngine {
    pub fn price(&mut self, item: &ManualDnsRecordReservation) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualDnsRecordReservationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualDnsRecordReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_dns_record_reservation_engine(flags: &flags::Registry) -> ManualDnsRecordReservationEngine {
    ManualDnsRecordReservationEngine {
        steps: vec![
            Rc::new(TieredManualDnsRecordReservationAdjuster {
                bounds: vec![5357.0, 9987.0],
                rates: vec![0.9, 0.86],
            }) as Rc<dyn ManualDnsRecordReservationStep>,
            Rc::new(FlaggedManualDnsRecordReservationAdjuster {
                flags: flags.clone(),
                flag: "manual-dns-record-reservation-pending".to_string(),
                percent: -4.1,
            }),
            Rc::new(PercentManualDnsRecordReservationAdjuster { percent: 16.0 }),
            Rc::new(MinimumManualDnsRecordReservationAdjuster { minimum: 27.5 }),
            Rc::new(ManualDnsRecordReservationThresholdOffsetAdjuster { weight: 0.77 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 63969.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_dns_record_reservation_engine(&flags::new_registry());
        let item = new_manual_dns_record_reservation("code-683", "title-964");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
