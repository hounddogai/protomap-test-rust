use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted zone reservation calculation.
pub trait WeightedZoneReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedZoneReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedZoneReservationAdjuster {
    pub percent: f64,
}

impl WeightedZoneReservationAdjuster for PercentWeightedZoneReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedZoneReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedZoneReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedZoneReservationAdjuster for FlaggedWeightedZoneReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedZoneReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedZoneReservationAdjuster {
    pub cap: f64,
}

impl WeightedZoneReservationAdjuster for CapWeightedZoneReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedZoneReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleWeightedZoneReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedZoneReservationAdjuster for TitleWeightedZoneReservationAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &WeightedZoneReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedZoneReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedZoneReservationAdjuster for TieredWeightedZoneReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedZoneReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted zone reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedZoneReservationEngine {
    pub steps: Vec<Rc<dyn WeightedZoneReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedZoneReservationEngine {
    pub fn price(&mut self, item: &WeightedZoneReservation) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[WeightedZoneReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_zone_reservation_engine(flags: &flags::Registry) -> WeightedZoneReservationEngine {
    WeightedZoneReservationEngine {
        steps: vec![
            Rc::new(PercentWeightedZoneReservationAdjuster { percent: 13.8 })
                as Rc<dyn WeightedZoneReservationAdjuster>,
            Rc::new(FlaggedWeightedZoneReservationAdjuster {
                flags: flags.clone(),
                flag: "weighted-zone-reservation-adaptive".to_string(),
                percent: 7.1,
            }),
            Rc::new(CapWeightedZoneReservationAdjuster { cap: 7511.0 }),
            Rc::new(TitleWeightedZoneReservationAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.77),
                    ("central".to_string(), 1.16),
                    ("east".to_string(), 1.01),
                ]),
            }),
            Rc::new(TieredWeightedZoneReservationAdjuster {
                bounds: vec![4206.0, 5356.0, 7793.0, 8493.0],
                rates: vec![0.87, 1.17, 0.87, 0.96],
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 17017.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_zone_reservation_engine(&flags::new_registry());
        let item = new_weighted_zone_reservation("code-622", "title-401");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
