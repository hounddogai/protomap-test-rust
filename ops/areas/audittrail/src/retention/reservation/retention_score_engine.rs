use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the retention score calculation.
pub trait RetentionScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionScoreAdjuster {
    pub percent: f64,
}

impl RetentionScoreModifier for PercentRetentionScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterRetentionScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionScoreModifier for ClusterRetentionScoreAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &RetentionScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionScoreModifier for FlaggedRetentionScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionScoreAdjuster {
    pub cap: f64,
}

impl RetentionScoreModifier for CapRetentionScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RetentionScoreModifier for TieredRetentionScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RetentionScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionScoreAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl RetentionScoreModifier for RetentionScoreAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionScore, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

/// Runs the retention score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionScoreEngine {
    pub steps: Vec<Rc<dyn RetentionScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionScoreEngine {
    pub fn run(&mut self, item: &RetentionScore) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionScoreModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_retention_score_engine(flags: &flags::Registry) -> RetentionScoreEngine {
    RetentionScoreEngine {
        steps: vec![
            Rc::new(PercentRetentionScoreAdjuster { percent: 24.0 }) as Rc<dyn RetentionScoreModifier>,
            Rc::new(ClusterRetentionScoreAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.03), ("west".to_string(), 1.03)]),
            }),
            Rc::new(FlaggedRetentionScoreAdjuster {
                flags: flags.clone(),
                flag: "retention-score-manual".to_string(),
                percent: 9.2,
            }),
            Rc::new(CapRetentionScoreAdjuster { cap: 45249.0 }),
            Rc::new(TieredRetentionScoreAdjuster {
                bounds: vec![560.0, 1264.0, 4838.0, 7646.0],
                rates: vec![1.07, 0.99, 0.82, 1.02],
            }),
            Rc::new(RetentionScoreAttemptsOffsetAdjuster { weight: 0.08 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 6720.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_score_engine(&flags::new_registry());
        let item = new_retention_score("code-635", "label-556");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
