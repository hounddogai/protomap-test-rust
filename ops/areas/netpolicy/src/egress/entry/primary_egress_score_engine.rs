use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary egress score calculation.
pub trait PrimaryEgressScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryEgressScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryEgressScoreAdjuster {
    pub percent: f64,
}

impl PrimaryEgressScoreModifier for PercentPrimaryEgressScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryEgressScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePrimaryEgressScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryEgressScoreModifier for TitlePrimaryEgressScoreAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PrimaryEgressScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryEgressScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryEgressScoreModifier for TieredPrimaryEgressScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryEgressScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryEgressScoreAdjuster {
    pub minimum: f64,
}

impl PrimaryEgressScoreModifier for MinimumPrimaryEgressScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryEgressScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryEgressScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryEgressScoreModifier for FlaggedPrimaryEgressScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryEgressScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryEgressScoreAdjuster {
    pub step: f64,
}

impl PrimaryEgressScoreModifier for RoundingPrimaryEgressScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryEgressScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the primary egress score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryEgressScoreEngine {
    pub steps: Vec<Rc<dyn PrimaryEgressScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryEgressScoreEngine {
    pub fn compute(&mut self, item: &PrimaryEgressScore) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryEgressScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_egress_score_engine(flags: &flags::Registry) -> PrimaryEgressScoreEngine {
    PrimaryEgressScoreEngine {
        steps: vec![
            Rc::new(PercentPrimaryEgressScoreAdjuster { percent: -13.5 }) as Rc<dyn PrimaryEgressScoreModifier>,
            Rc::new(TitlePrimaryEgressScoreAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.22),
                    ("north".to_string(), 0.74),
                    ("east".to_string(), 0.9),
                ]),
            }),
            Rc::new(TieredPrimaryEgressScoreAdjuster {
                bounds: vec![1100.0, 2550.0, 3064.0, 6931.0],
                rates: vec![0.95, 0.84, 1.06, 0.88],
            }),
            Rc::new(MinimumPrimaryEgressScoreAdjuster { minimum: 3.5 }),
            Rc::new(FlaggedPrimaryEgressScoreAdjuster {
                flags: flags.clone(),
                flag: "primary-egress-score-tiered".to_string(),
                percent: -4.3,
            }),
            Rc::new(RoundingPrimaryEgressScoreAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 72472.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_egress_score_engine(&flags::new_registry());
        let item = new_primary_egress_score("code-542", "title-105");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
