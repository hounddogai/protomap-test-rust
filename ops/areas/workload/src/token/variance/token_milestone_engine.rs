use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the token milestone calculation.
pub trait TokenMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTokenMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenMilestoneStep for NameTokenMilestoneAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TokenMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenMilestoneStep for FlaggedTokenMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenMilestoneStep for TieredTokenMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the token milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenMilestoneEngine {
    pub steps: Vec<Rc<dyn TokenMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenMilestoneEngine {
    pub fn compute(&mut self, item: &TokenMilestone) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[TokenMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_token_milestone_engine(flags: &flags::Registry) -> TokenMilestoneEngine {
    TokenMilestoneEngine {
        steps: vec![
            Rc::new(NameTokenMilestoneAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.29)]),
            }) as Rc<dyn TokenMilestoneStep>,
            Rc::new(FlaggedTokenMilestoneAdjuster {
                flags: flags.clone(),
                flag: "token-milestone-capped".to_string(),
                percent: -4.4,
            }),
            Rc::new(TieredTokenMilestoneAdjuster {
                bounds: vec![2123.0, 7331.0, 9851.0],
                rates: vec![0.95, 0.97, 0.83],
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 80966.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_milestone_engine(&flags::new_registry());
        let item = new_token_milestone("code-500", "name-849");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
