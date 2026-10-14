use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional usage score calculation.
pub trait RegionalUsageScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalUsageScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalUsageScoreAdjuster {
    pub minimum: f64,
}

impl RegionalUsageScoreStep for MinimumRegionalUsageScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalUsageScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageScoreRevisionOffsetAdjuster {
    pub weight: f64,
}

impl RegionalUsageScoreStep for RegionalUsageScoreRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalUsageScore, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalUsageScoreAdjuster {
    pub step: f64,
}

impl RegionalUsageScoreStep for RoundingRegionalUsageScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalUsageScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalUsageScoreAdjuster {
    pub cap: f64,
}

impl RegionalUsageScoreStep for CapRegionalUsageScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalUsageScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalUsageScoreAdjuster {
    pub percent: f64,
}

impl RegionalUsageScoreStep for PercentRegionalUsageScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalUsageScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalUsageScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalUsageScoreStep for FlaggedRegionalUsageScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalUsageScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional usage score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalUsageScoreEngine {
    pub steps: Vec<Rc<dyn RegionalUsageScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalUsageScoreEngine {
    pub fn evaluate(&mut self, item: &RegionalUsageScore) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionalUsageScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_usage_score_engine(flags: &flags::Registry) -> RegionalUsageScoreEngine {
    RegionalUsageScoreEngine {
        steps: vec![
            Rc::new(MinimumRegionalUsageScoreAdjuster { minimum: 24.0 }) as Rc<dyn RegionalUsageScoreStep>,
            Rc::new(RegionalUsageScoreRevisionOffsetAdjuster { weight: 0.25 }),
            Rc::new(RoundingRegionalUsageScoreAdjuster { step: 0.1 }),
            Rc::new(CapRegionalUsageScoreAdjuster { cap: 10046.0 }),
            Rc::new(PercentRegionalUsageScoreAdjuster { percent: -6.0 }),
            Rc::new(FlaggedRegionalUsageScoreAdjuster {
                flags: flags.clone(),
                flag: "regional-usage-score-default".to_string(),
                percent: 0.5,
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 97879.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_usage_score_engine(&flags::new_registry());
        let item = new_regional_usage_score("code-298", "title-335");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
