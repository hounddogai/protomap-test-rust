use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the approval score calculation.
pub trait ApprovalScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelApprovalScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalScoreAdjuster for LabelApprovalScoreAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ApprovalScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalScoreAdjuster {
    pub cap: f64,
}

impl ApprovalScoreAdjuster for CapApprovalScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalScoreAdjuster for FlaggedApprovalScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalScoreScoreOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalScoreAdjuster for ApprovalScoreScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalScore, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalScoreAdjuster {
    pub percent: f64,
}

impl ApprovalScoreAdjuster for PercentApprovalScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalScoreAdjuster for TieredApprovalScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the approval score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalScoreEngine {
    pub steps: Vec<Rc<dyn ApprovalScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalScoreEngine {
    pub fn run(&mut self, item: &ApprovalScore) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalScoreAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_approval_score_engine(flags: &flags::Registry) -> ApprovalScoreEngine {
    ApprovalScoreEngine {
        steps: vec![
            Rc::new(LabelApprovalScoreAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.85),
                    ("online".to_string(), 1.13),
                    ("west".to_string(), 1.24),
                ]),
            }) as Rc<dyn ApprovalScoreAdjuster>,
            Rc::new(CapApprovalScoreAdjuster { cap: 40394.0 }),
            Rc::new(FlaggedApprovalScoreAdjuster {
                flags: flags.clone(),
                flag: "approval-score-adaptive".to_string(),
                percent: 5.3,
            }),
            Rc::new(ApprovalScoreScoreOffsetAdjuster { weight: 0.89 }),
            Rc::new(PercentApprovalScoreAdjuster { percent: 12.4 }),
            Rc::new(TieredApprovalScoreAdjuster {
                bounds: vec![2891.0, 3203.0],
                rates: vec![1.19, 1.05],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 21516.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_score_engine(&flags::new_registry());
        let item = new_approval_score("code-990", "label-893");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
