use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict limit variance calculation.
pub trait StrictLimitVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictLimitVariance, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictLimitVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictLimitVarianceAdjuster for FlaggedStrictLimitVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictLimitVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictLimitVarianceAdjuster {
    pub step: f64,
}

impl StrictLimitVarianceAdjuster for RoundingStrictLimitVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictLimitVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleStrictLimitVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictLimitVarianceAdjuster for TitleStrictLimitVarianceAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &StrictLimitVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictLimitVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictLimitVarianceAdjuster for TieredStrictLimitVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictLimitVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the strict limit variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictLimitVarianceEngine {
    pub steps: Vec<Rc<dyn StrictLimitVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictLimitVarianceEngine {
    pub fn evaluate(&mut self, item: &StrictLimitVariance) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictLimitVarianceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictLimitVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_limit_variance_engine(flags: &flags::Registry) -> StrictLimitVarianceEngine {
    StrictLimitVarianceEngine {
        steps: vec![
            Rc::new(FlaggedStrictLimitVarianceAdjuster {
                flags: flags.clone(),
                flag: "strict-limit-variance-rolling".to_string(),
                percent: -3.2,
            }) as Rc<dyn StrictLimitVarianceAdjuster>,
            Rc::new(RoundingStrictLimitVarianceAdjuster { step: 5.0 }),
            Rc::new(TitleStrictLimitVarianceAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.93), ("west".to_string(), 1.27)]),
            }),
            Rc::new(TieredStrictLimitVarianceAdjuster {
                bounds: vec![653.0, 2272.0, 4183.0],
                rates: vec![0.92, 1.17, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 68401.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_limit_variance_engine(&flags::new_registry());
        let item = new_strict_limit_variance("code-136", "title-999");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
