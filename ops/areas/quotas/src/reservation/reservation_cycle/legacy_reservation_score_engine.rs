use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy reservation score calculation.
pub trait LegacyReservationScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyReservationScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyReservationScoreAdjuster {
    pub cap: f64,
}

impl LegacyReservationScoreModifier for CapLegacyReservationScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyReservationScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyReservationScoreAdjuster {
    pub percent: f64,
}

impl LegacyReservationScoreModifier for PercentLegacyReservationScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyReservationScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationScoreBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl LegacyReservationScoreModifier for LegacyReservationScoreBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyReservationScore, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyReservationScoreAdjuster {
    pub step: f64,
}

impl LegacyReservationScoreModifier for RoundingLegacyReservationScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyReservationScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyReservationScoreAdjuster {
    pub minimum: f64,
}

impl LegacyReservationScoreModifier for MinimumLegacyReservationScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyReservationScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy reservation score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyReservationScoreEngine {
    pub steps: Vec<Rc<dyn LegacyReservationScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyReservationScoreEngine {
    pub fn evaluate(&mut self, item: &LegacyReservationScore) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[LegacyReservationScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_reservation_score_engine() -> LegacyReservationScoreEngine {
    LegacyReservationScoreEngine {
        steps: vec![
            Rc::new(CapLegacyReservationScoreAdjuster { cap: 32926.0 }) as Rc<dyn LegacyReservationScoreModifier>,
            Rc::new(PercentLegacyReservationScoreAdjuster { percent: -22.2 }),
            Rc::new(LegacyReservationScoreBatchSizeOffsetAdjuster { weight: 0.78 }),
            Rc::new(RoundingLegacyReservationScoreAdjuster { step: 1.0 }),
            Rc::new(MinimumLegacyReservationScoreAdjuster { minimum: 21.8 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 1694.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_reservation_score_engine();
        let item = new_legacy_reservation_score("code-309", "title-786");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
