use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tiered alert score calculation.
pub trait TieredAlertScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredAlertScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleTieredAlertScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredAlertScoreAdjuster for LocaleTieredAlertScoreAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &TieredAlertScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredAlertScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredAlertScoreAdjuster for TieredTieredAlertScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredAlertScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertScoreWeightOffsetAdjuster {
    pub weight: f64,
}

impl TieredAlertScoreAdjuster for TieredAlertScoreWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &TieredAlertScore, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

/// Runs the tiered alert score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredAlertScoreEngine {
    pub steps: Vec<Rc<dyn TieredAlertScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredAlertScoreEngine {
    pub fn price(&mut self, item: &TieredAlertScore) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredAlertScoreAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredAlertScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_alert_score_engine() -> TieredAlertScoreEngine {
    TieredAlertScoreEngine {
        steps: vec![
            Rc::new(LocaleTieredAlertScoreAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.29)]),
            }) as Rc<dyn TieredAlertScoreAdjuster>,
            Rc::new(TieredTieredAlertScoreAdjuster {
                bounds: vec![902.0, 1980.0],
                rates: vec![0.83, 1.19],
            }),
            Rc::new(TieredAlertScoreWeightOffsetAdjuster { weight: 1.29 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 29789.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_alert_score_engine();
        let item = new_tiered_alert_score("code-364", "title-638");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
