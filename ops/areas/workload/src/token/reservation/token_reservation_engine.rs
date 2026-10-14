use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the token reservation calculation.
pub trait TokenReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenReservationScoreOffsetAdjuster {
    pub weight: f64,
}

impl TokenReservationAdjuster for TokenReservationScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &TokenReservation, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenReservationAdjuster for FlaggedTokenReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenReservationAdjuster {
    pub minimum: f64,
}

impl TokenReservationAdjuster for MinimumTokenReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the token reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenReservationEngine {
    pub steps: Vec<Rc<dyn TokenReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenReservationEngine {
    pub fn price(&mut self, item: &TokenReservation) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TokenReservationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TokenReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_token_reservation_engine(flags: &flags::Registry) -> TokenReservationEngine {
    TokenReservationEngine {
        steps: vec![
            Rc::new(TokenReservationScoreOffsetAdjuster { weight: 1.69 }) as Rc<dyn TokenReservationAdjuster>,
            Rc::new(FlaggedTokenReservationAdjuster {
                flags: flags.clone(),
                flag: "token-reservation-rolling".to_string(),
                percent: 9.7,
            }),
            Rc::new(MinimumTokenReservationAdjuster { minimum: 27.5 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 38167.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_reservation_engine(&flags::new_registry());
        let item = new_token_reservation("code-369", "title-702");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
