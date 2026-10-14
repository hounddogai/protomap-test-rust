use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the regional failover variance calculation.
pub trait RegionalFailoverVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalFailoverVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeRegionalFailoverVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalFailoverVarianceAdjuster for ScopeRegionalFailoverVarianceAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &RegionalFailoverVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalFailoverVarianceAdjuster {
    pub percent: f64,
}

impl RegionalFailoverVarianceAdjuster for PercentRegionalFailoverVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalFailoverVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverVarianceMarginOffsetAdjuster {
    pub weight: f64,
}

impl RegionalFailoverVarianceAdjuster for RegionalFailoverVarianceMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalFailoverVariance, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the regional failover variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalFailoverVarianceEngine {
    pub steps: Vec<Rc<dyn RegionalFailoverVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalFailoverVarianceEngine {
    pub fn price(&mut self, item: &RegionalFailoverVariance) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalFailoverVarianceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalFailoverVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_failover_variance_engine() -> RegionalFailoverVarianceEngine {
    RegionalFailoverVarianceEngine {
        steps: vec![
            Rc::new(ScopeRegionalFailoverVarianceAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.71),
                    ("north".to_string(), 0.88),
                    ("south".to_string(), 1.12),
                ]),
            }) as Rc<dyn RegionalFailoverVarianceAdjuster>,
            Rc::new(PercentRegionalFailoverVarianceAdjuster { percent: 5.0 }),
            Rc::new(RegionalFailoverVarianceMarginOffsetAdjuster { weight: 1.16 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 78814.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_failover_variance_engine();
        let item = new_regional_failover_variance("code-144", "name-653");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
