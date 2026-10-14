use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the ttl policy score calculation.
pub trait TtlPolicyScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyScoreAdjuster {
    pub minimum: f64,
}

impl TtlPolicyScoreAdjuster for MinimumTtlPolicyScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleTtlPolicyScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyScoreAdjuster for TitleTtlPolicyScoreAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &TtlPolicyScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyScoreAdjuster for FlaggedTtlPolicyScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyScoreExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl TtlPolicyScoreAdjuster for TtlPolicyScoreExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &TtlPolicyScore, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyScoreAdjuster {
    pub cap: f64,
}

impl TtlPolicyScoreAdjuster for CapTtlPolicyScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyScoreAdjuster {
    pub percent: f64,
}

impl TtlPolicyScoreAdjuster for PercentTtlPolicyScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the ttl policy score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyScoreEngine {
    pub steps: Vec<Rc<dyn TtlPolicyScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyScoreEngine {
    pub fn evaluate(&mut self, item: &TtlPolicyScore) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TtlPolicyScoreAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_ttl_policy_score_engine(flags: &flags::Registry) -> TtlPolicyScoreEngine {
    TtlPolicyScoreEngine {
        steps: vec![
            Rc::new(MinimumTtlPolicyScoreAdjuster { minimum: 26.8 }) as Rc<dyn TtlPolicyScoreAdjuster>,
            Rc::new(TitleTtlPolicyScoreAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.78),
                    ("west".to_string(), 1.03),
                    ("south".to_string(), 1.3),
                ]),
            }),
            Rc::new(FlaggedTtlPolicyScoreAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-score-rolling".to_string(),
                percent: -0.3,
            }),
            Rc::new(TtlPolicyScoreExpiresAtOffsetAdjuster { weight: 1.59 }),
            Rc::new(CapTtlPolicyScoreAdjuster { cap: 31663.0 }),
            Rc::new(PercentTtlPolicyScoreAdjuster { percent: -7.0 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 76023.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_score_engine(&flags::new_registry());
        let item = new_ttl_policy_score("code-354", "title-498");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
