use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional query profile calculation.
pub trait RegionalQueryProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalQueryProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalQueryProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalQueryProfileModifier for TieredRegionalQueryProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalQueryProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalQueryProfileAdjuster {
    pub step: f64,
}

impl RegionalQueryProfileModifier for RoundingRegionalQueryProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalQueryProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRegionalQueryProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalQueryProfileModifier for CategoryRegionalQueryProfileAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RegionalQueryProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalQueryProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalQueryProfileModifier for FlaggedRegionalQueryProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalQueryProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalQueryProfileAdjuster {
    pub minimum: f64,
}

impl RegionalQueryProfileModifier for MinimumRegionalQueryProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalQueryProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional query profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalQueryProfileEngine {
    pub steps: Vec<Rc<dyn RegionalQueryProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalQueryProfileEngine {
    pub fn evaluate(&mut self, item: &RegionalQueryProfile) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalQueryProfileModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalQueryProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_query_profile_engine(flags: &flags::Registry) -> RegionalQueryProfileEngine {
    RegionalQueryProfileEngine {
        steps: vec![
            Rc::new(TieredRegionalQueryProfileAdjuster {
                bounds: vec![386.0, 456.0, 4441.0, 5571.0],
                rates: vec![0.97, 0.91, 1.02, 0.89],
            }) as Rc<dyn RegionalQueryProfileModifier>,
            Rc::new(RoundingRegionalQueryProfileAdjuster { step: 0.25 }),
            Rc::new(CategoryRegionalQueryProfileAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.27), ("west".to_string(), 1.14)]),
            }),
            Rc::new(FlaggedRegionalQueryProfileAdjuster {
                flags: flags.clone(),
                flag: "regional-query-profile-global".to_string(),
                percent: -2.7,
            }),
            Rc::new(MinimumRegionalQueryProfileAdjuster { minimum: 46.9 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 42606.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_query_profile_engine(&flags::new_registry());
        let item = new_regional_query_profile("code-36", "name-276");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
