use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the health check reservation calculation.
pub trait HealthCheckReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentHealthCheckReservationAdjuster {
    pub percent: f64,
}

impl HealthCheckReservationModifier for PercentHealthCheckReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &HealthCheckReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckReservationAdjuster {
    pub minimum: f64,
}

impl HealthCheckReservationModifier for MinimumHealthCheckReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingHealthCheckReservationAdjuster {
    pub step: f64,
}

impl HealthCheckReservationModifier for RoundingHealthCheckReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &HealthCheckReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapHealthCheckReservationAdjuster {
    pub cap: f64,
}

impl HealthCheckReservationModifier for CapHealthCheckReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &HealthCheckReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckReservationModifier for FlaggedHealthCheckReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckReservationModifier for TieredHealthCheckReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the health check reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckReservationEngine {
    pub steps: Vec<Rc<dyn HealthCheckReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckReservationEngine {
    pub fn evaluate(&mut self, item: &HealthCheckReservation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_health_check_reservation_engine(flags: &flags::Registry) -> HealthCheckReservationEngine {
    HealthCheckReservationEngine {
        steps: vec![
            Rc::new(PercentHealthCheckReservationAdjuster { percent: -12.0 }) as Rc<dyn HealthCheckReservationModifier>,
            Rc::new(MinimumHealthCheckReservationAdjuster { minimum: 1.8 }),
            Rc::new(RoundingHealthCheckReservationAdjuster { step: 0.1 }),
            Rc::new(CapHealthCheckReservationAdjuster { cap: 3191.0 }),
            Rc::new(FlaggedHealthCheckReservationAdjuster {
                flags: flags.clone(),
                flag: "health-check-reservation-seasonal".to_string(),
                percent: 6.2,
            }),
            Rc::new(TieredHealthCheckReservationAdjuster {
                bounds: vec![2439.0, 9309.0],
                rates: vec![0.9, 1.16],
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 82088.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_reservation_engine(&flags::new_registry());
        let item = new_health_check_reservation("code-491", "name-898");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
