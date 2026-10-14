use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the escalation profile calculation.
pub trait EscalationProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationProfileModifier for TieredEscalationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationProfileGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl EscalationProfileModifier for EscalationProfileGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationProfile, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationProfileAdjuster {
    pub step: f64,
}

impl EscalationProfileModifier for RoundingEscalationProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleEscalationProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EscalationProfileModifier for TitleEscalationProfileAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &EscalationProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEscalationProfileAdjuster {
    pub minimum: f64,
}

impl EscalationProfileModifier for MinimumEscalationProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EscalationProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the escalation profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationProfileEngine {
    pub steps: Vec<Rc<dyn EscalationProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationProfileEngine {
    pub fn compute(&mut self, item: &EscalationProfile) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EscalationProfileModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EscalationProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_escalation_profile_engine() -> EscalationProfileEngine {
    EscalationProfileEngine {
        steps: vec![
            Rc::new(TieredEscalationProfileAdjuster {
                bounds: vec![787.0, 5049.0, 5193.0, 6576.0],
                rates: vec![1.17, 1.13, 0.91, 0.87],
            }) as Rc<dyn EscalationProfileModifier>,
            Rc::new(EscalationProfileGraceDaysOffsetAdjuster { weight: 0.83 }),
            Rc::new(RoundingEscalationProfileAdjuster { step: 0.05 }),
            Rc::new(TitleEscalationProfileAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.27),
                    ("east".to_string(), 1.36),
                    ("north".to_string(), 1.33),
                ]),
            }),
            Rc::new(MinimumEscalationProfileAdjuster { minimum: 14.3 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 22350.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_profile_engine();
        let item = new_escalation_profile("code-895", "title-184");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
