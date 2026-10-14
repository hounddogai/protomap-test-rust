use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the probe reservation calculation.
pub trait ProbeReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbeReservationModifier for TieredProbeReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbeReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingProbeReservationAdjuster {
    pub step: f64,
}

impl ProbeReservationModifier for RoundingProbeReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ProbeReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeReservationAdjuster {
    pub percent: f64,
}

impl ProbeReservationModifier for PercentProbeReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedProbeReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ProbeReservationModifier for FlaggedProbeReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ProbeReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryProbeReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ProbeReservationModifier for CountryProbeReservationAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ProbeReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the probe reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeReservationEngine {
    pub steps: Vec<Rc<dyn ProbeReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeReservationEngine {
    pub fn run(&mut self, item: &ProbeReservation) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[ProbeReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_probe_reservation_engine(flags: &flags::Registry) -> ProbeReservationEngine {
    ProbeReservationEngine {
        steps: vec![
            Rc::new(TieredProbeReservationAdjuster {
                bounds: vec![1206.0, 2769.0, 7885.0],
                rates: vec![0.84, 0.9, 1.06],
            }) as Rc<dyn ProbeReservationModifier>,
            Rc::new(RoundingProbeReservationAdjuster { step: 1.0 }),
            Rc::new(PercentProbeReservationAdjuster { percent: -23.9 }),
            Rc::new(FlaggedProbeReservationAdjuster {
                flags: flags.clone(),
                flag: "probe-reservation-primary".to_string(),
                percent: -6.8,
            }),
            Rc::new(CountryProbeReservationAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.24),
                    ("online".to_string(), 0.84),
                    ("retail".to_string(), 1.12),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 77208.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_reservation_engine(&flags::new_registry());
        let item = new_probe_reservation("code-401", "label-55");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
