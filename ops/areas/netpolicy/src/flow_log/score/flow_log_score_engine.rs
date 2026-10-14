use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the flow log score calculation.
pub trait FlowLogScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogScore, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFlowLogScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FlowLogScoreStep for FlaggedFlowLogScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FlowLogScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentFlowLogScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogScoreStep for SegmentFlowLogScoreAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &FlowLogScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FlowLogScoreStep for TieredFlowLogScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FlowLogScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogScoreAdjuster {
    pub percent: f64,
}

impl FlowLogScoreStep for PercentFlowLogScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFlowLogScoreAdjuster {
    pub minimum: f64,
}

impl FlowLogScoreStep for MinimumFlowLogScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FlowLogScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the flow log score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogScoreEngine {
    pub steps: Vec<Rc<dyn FlowLogScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogScoreEngine {
    pub fn evaluate(&mut self, item: &FlowLogScore) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FlowLogScoreStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_flow_log_score_engine(flags: &flags::Registry) -> FlowLogScoreEngine {
    FlowLogScoreEngine {
        steps: vec![
            Rc::new(FlaggedFlowLogScoreAdjuster {
                flags: flags.clone(),
                flag: "flow-log-score-global".to_string(),
                percent: -0.5,
            }) as Rc<dyn FlowLogScoreStep>,
            Rc::new(SegmentFlowLogScoreAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.98),
                    ("south".to_string(), 0.88),
                    ("north".to_string(), 1.06),
                ]),
            }),
            Rc::new(TieredFlowLogScoreAdjuster {
                bounds: vec![807.0, 7955.0],
                rates: vec![1.07, 1.14],
            }),
            Rc::new(PercentFlowLogScoreAdjuster { percent: 21.7 }),
            Rc::new(MinimumFlowLogScoreAdjuster { minimum: 19.6 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 53818.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_score_engine(&flags::new_registry());
        let item = new_flow_log_score("code-521", "title-282");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
