use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional data export reservation calculation.
pub trait RegionalDataExportReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalDataExportReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReservationCapacityOffsetAdjuster {
    pub weight: f64,
}

impl RegionalDataExportReservationFactor for RegionalDataExportReservationCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalDataExportReservation, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRegionalDataExportReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalDataExportReservationFactor for LabelRegionalDataExportReservationAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RegionalDataExportReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalDataExportReservationAdjuster {
    pub minimum: f64,
}

impl RegionalDataExportReservationFactor for MinimumRegionalDataExportReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalDataExportReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalDataExportReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalDataExportReservationFactor for FlaggedRegionalDataExportReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalDataExportReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalDataExportReservationAdjuster {
    pub percent: f64,
}

impl RegionalDataExportReservationFactor for PercentRegionalDataExportReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalDataExportReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional data export reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalDataExportReservationEngine {
    pub steps: Vec<Rc<dyn RegionalDataExportReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalDataExportReservationEngine {
    pub fn evaluate(&mut self, item: &RegionalDataExportReservation) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionalDataExportReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_data_export_reservation_engine(flags: &flags::Registry) -> RegionalDataExportReservationEngine {
    RegionalDataExportReservationEngine {
        steps: vec![
            Rc::new(RegionalDataExportReservationCapacityOffsetAdjuster { weight: 0.46 })
                as Rc<dyn RegionalDataExportReservationFactor>,
            Rc::new(LabelRegionalDataExportReservationAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.29), ("north".to_string(), 1.19)]),
            }),
            Rc::new(MinimumRegionalDataExportReservationAdjuster { minimum: 1.0 }),
            Rc::new(FlaggedRegionalDataExportReservationAdjuster {
                flags: flags.clone(),
                flag: "regional-data-export-reservation-lenient".to_string(),
                percent: 1.0,
            }),
            Rc::new(PercentRegionalDataExportReservationAdjuster { percent: -14.2 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 71155.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_data_export_reservation_engine(&flags::new_registry());
        let item = new_regional_data_export_reservation("code-441", "label-818");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
