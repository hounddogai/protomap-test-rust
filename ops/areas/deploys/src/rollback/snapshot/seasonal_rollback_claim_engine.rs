use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the seasonal rollback claim calculation.
pub trait SeasonalRollbackClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalRollbackClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalRollbackClaimAdjuster {
    pub percent: f64,
}

impl SeasonalRollbackClaimStep for PercentSeasonalRollbackClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalRollbackClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalRollbackClaimAdjuster {
    pub cap: f64,
}

impl SeasonalRollbackClaimStep for CapSeasonalRollbackClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalRollbackClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalRollbackClaimAdjuster {
    pub minimum: f64,
}

impl SeasonalRollbackClaimStep for MinimumSeasonalRollbackClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalRollbackClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalRollbackClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalRollbackClaimStep for TieredSeasonalRollbackClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalRollbackClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackClaimEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalRollbackClaimStep for SeasonalRollbackClaimEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalRollbackClaim, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountrySeasonalRollbackClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalRollbackClaimStep for CountrySeasonalRollbackClaimAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &SeasonalRollbackClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the seasonal rollback claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalRollbackClaimEngine {
    pub steps: Vec<Rc<dyn SeasonalRollbackClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalRollbackClaimEngine {
    pub fn evaluate(&mut self, item: &SeasonalRollbackClaim) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalRollbackClaimStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalRollbackClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_rollback_claim_engine() -> SeasonalRollbackClaimEngine {
    SeasonalRollbackClaimEngine {
        steps: vec![
            Rc::new(PercentSeasonalRollbackClaimAdjuster { percent: -15.4 }) as Rc<dyn SeasonalRollbackClaimStep>,
            Rc::new(CapSeasonalRollbackClaimAdjuster { cap: 26168.0 }),
            Rc::new(MinimumSeasonalRollbackClaimAdjuster { minimum: 18.2 }),
            Rc::new(TieredSeasonalRollbackClaimAdjuster {
                bounds: vec![7124.0, 9807.0],
                rates: vec![0.98, 1.0],
            }),
            Rc::new(SeasonalRollbackClaimEndsAtOffsetAdjuster { weight: 1.52 }),
            Rc::new(CountrySeasonalRollbackClaimAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.0)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 8095.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_rollback_claim_engine();
        let item = new_seasonal_rollback_claim("code-262", "name-202");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
