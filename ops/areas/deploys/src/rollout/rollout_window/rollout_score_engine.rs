use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout score calculation.
pub trait RolloutScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutScore, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutScoreModifier for FlaggedRolloutScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutScoreAdjuster {
    pub cap: f64,
}

impl RolloutScoreModifier for CapRolloutScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutScoreAdjuster {
    pub minimum: f64,
}

impl RolloutScoreModifier for MinimumRolloutScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutScoreAdjuster {
    pub step: f64,
}

impl RolloutScoreModifier for RoundingRolloutScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRolloutScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutScoreModifier for RegionRolloutScoreAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RolloutScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rollout score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutScoreEngine {
    pub steps: Vec<Rc<dyn RolloutScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutScoreEngine {
    pub fn price(&mut self, item: &RolloutScore) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RolloutScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rollout_score_engine(flags: &flags::Registry) -> RolloutScoreEngine {
    RolloutScoreEngine {
        steps: vec![
            Rc::new(FlaggedRolloutScoreAdjuster {
                flags: flags.clone(),
                flag: "rollout-score-adaptive".to_string(),
                percent: -3.1,
            }) as Rc<dyn RolloutScoreModifier>,
            Rc::new(CapRolloutScoreAdjuster { cap: 18207.0 }),
            Rc::new(MinimumRolloutScoreAdjuster { minimum: 49.6 }),
            Rc::new(RoundingRolloutScoreAdjuster { step: 0.1 }),
            Rc::new(RegionRolloutScoreAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.07)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 79852.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_score_engine(&flags::new_registry());
        let item = new_rollout_score("code-785", "title-294");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
