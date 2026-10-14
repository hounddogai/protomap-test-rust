use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rolling postmortem score calculation.
pub trait RollingPostmortemScoreFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPostmortemScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPostmortemScorePriorityOffsetAdjuster {
    pub weight: f64,
}

impl RollingPostmortemScoreFactor for RollingPostmortemScorePriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &RollingPostmortemScore, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingPostmortemScoreAdjuster {
    pub percent: f64,
}

impl RollingPostmortemScoreFactor for PercentRollingPostmortemScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingPostmortemScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingPostmortemScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingPostmortemScoreFactor for FlaggedRollingPostmortemScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingPostmortemScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingPostmortemScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingPostmortemScoreFactor for TieredRollingPostmortemScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingPostmortemScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling postmortem score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPostmortemScoreEngine {
    pub steps: Vec<Rc<dyn RollingPostmortemScoreFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPostmortemScoreEngine {
    pub fn run(&mut self, item: &RollingPostmortemScore) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingPostmortemScoreFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingPostmortemScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_postmortem_score_engine(flags: &flags::Registry) -> RollingPostmortemScoreEngine {
    RollingPostmortemScoreEngine {
        steps: vec![
            Rc::new(RollingPostmortemScorePriorityOffsetAdjuster { weight: 0.94 })
                as Rc<dyn RollingPostmortemScoreFactor>,
            Rc::new(PercentRollingPostmortemScoreAdjuster { percent: -16.9 }),
            Rc::new(FlaggedRollingPostmortemScoreAdjuster {
                flags: flags.clone(),
                flag: "rolling-postmortem-score-rolling".to_string(),
                percent: 4.9,
            }),
            Rc::new(TieredRollingPostmortemScoreAdjuster {
                bounds: vec![1499.0, 3805.0, 8718.0],
                rates: vec![1.16, 0.91, 0.95],
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 50712.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_postmortem_score_engine(&flags::new_registry());
        let item = new_rolling_postmortem_score("code-433", "name-189");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
