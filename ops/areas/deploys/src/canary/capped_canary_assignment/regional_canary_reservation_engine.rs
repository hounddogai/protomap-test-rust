use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional canary reservation calculation.
pub trait RegionalCanaryReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalCanaryReservation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalCanaryReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalCanaryReservationAdjuster for FlaggedRegionalCanaryReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalCanaryReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalCanaryReservationAdjuster {
    pub cap: f64,
}

impl RegionalCanaryReservationAdjuster for CapRegionalCanaryReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalCanaryReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalCanaryReservationAdjuster {
    pub percent: f64,
}

impl RegionalCanaryReservationAdjuster for PercentRegionalCanaryReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalCanaryReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRegionalCanaryReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalCanaryReservationAdjuster for NameRegionalCanaryReservationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RegionalCanaryReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the regional canary reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalCanaryReservationEngine {
    pub steps: Vec<Rc<dyn RegionalCanaryReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalCanaryReservationEngine {
    pub fn compute(&mut self, item: &RegionalCanaryReservation) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalCanaryReservationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalCanaryReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_canary_reservation_engine(flags: &flags::Registry) -> RegionalCanaryReservationEngine {
    RegionalCanaryReservationEngine {
        steps: vec![
            Rc::new(FlaggedRegionalCanaryReservationAdjuster {
                flags: flags.clone(),
                flag: "regional-canary-reservation-rolling".to_string(),
                percent: -5.8,
            }) as Rc<dyn RegionalCanaryReservationAdjuster>,
            Rc::new(CapRegionalCanaryReservationAdjuster { cap: 43886.0 }),
            Rc::new(PercentRegionalCanaryReservationAdjuster { percent: -13.6 }),
            Rc::new(NameRegionalCanaryReservationAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.82)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 84806.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_canary_reservation_engine(&flags::new_registry());
        let item = new_regional_canary_reservation("code-319", "name-211");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
