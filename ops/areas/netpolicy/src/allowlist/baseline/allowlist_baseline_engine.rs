use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the allowlist baseline calculation.
pub trait AllowlistBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistBaselineAdjuster {
    pub cap: f64,
}

impl AllowlistBaselineAdjuster for CapAllowlistBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleAllowlistBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AllowlistBaselineAdjuster for LocaleAllowlistBaselineAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &AllowlistBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAllowlistBaselineAdjuster {
    pub minimum: f64,
}

impl AllowlistBaselineAdjuster for MinimumAllowlistBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AllowlistBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBaselineAmountOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistBaselineAdjuster for AllowlistBaselineAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistBaseline, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AllowlistBaselineAdjuster for TieredAllowlistBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AllowlistBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the allowlist baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistBaselineEngine {
    pub steps: Vec<Rc<dyn AllowlistBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistBaselineEngine {
    pub fn run(&mut self, item: &AllowlistBaseline) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AllowlistBaselineAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_allowlist_baseline_engine() -> AllowlistBaselineEngine {
    AllowlistBaselineEngine {
        steps: vec![
            Rc::new(CapAllowlistBaselineAdjuster { cap: 48443.0 }) as Rc<dyn AllowlistBaselineAdjuster>,
            Rc::new(LocaleAllowlistBaselineAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.36)]),
            }),
            Rc::new(MinimumAllowlistBaselineAdjuster { minimum: 7.9 }),
            Rc::new(AllowlistBaselineAmountOffsetAdjuster { weight: 1.47 }),
            Rc::new(TieredAllowlistBaselineAdjuster {
                bounds: vec![2876.0, 6512.0],
                rates: vec![0.97, 0.83],
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 93495.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_allowlist_baseline_engine();
        let item = new_allowlist_baseline("code-45", "label-412");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
