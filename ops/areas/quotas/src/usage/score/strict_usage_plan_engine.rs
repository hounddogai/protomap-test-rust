use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict usage plan calculation.
pub trait StrictUsagePlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictUsagePlan, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictUsagePlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictUsagePlanAdjuster for FlaggedStrictUsagePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictUsagePlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictUsagePlanAdjuster {
    pub step: f64,
}

impl StrictUsagePlanAdjuster for RoundingStrictUsagePlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictUsagePlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictUsagePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictUsagePlanAdjuster for TieredStrictUsagePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictUsagePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictUsagePlanScoreOffsetAdjuster {
    pub weight: f64,
}

impl StrictUsagePlanAdjuster for StrictUsagePlanScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &StrictUsagePlan, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the strict usage plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictUsagePlanEngine {
    pub steps: Vec<Rc<dyn StrictUsagePlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictUsagePlanEngine {
    pub fn run(&mut self, item: &StrictUsagePlan) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictUsagePlanAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictUsagePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_usage_plan_engine(flags: &flags::Registry) -> StrictUsagePlanEngine {
    StrictUsagePlanEngine {
        steps: vec![
            Rc::new(FlaggedStrictUsagePlanAdjuster {
                flags: flags.clone(),
                flag: "strict-usage-plan-manual".to_string(),
                percent: 4.4,
            }) as Rc<dyn StrictUsagePlanAdjuster>,
            Rc::new(RoundingStrictUsagePlanAdjuster { step: 1.0 }),
            Rc::new(TieredStrictUsagePlanAdjuster {
                bounds: vec![2305.0, 3020.0],
                rates: vec![1.11, 0.89],
            }),
            Rc::new(StrictUsagePlanScoreOffsetAdjuster { weight: 1.46 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 72680.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_usage_plan_engine(&flags::new_registry());
        let item = new_strict_usage_plan("code-31", "name-22");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
