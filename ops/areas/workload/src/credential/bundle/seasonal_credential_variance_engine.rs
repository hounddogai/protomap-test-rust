use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal credential variance calculation.
pub trait SeasonalCredentialVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalCredentialVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeSeasonalCredentialVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalCredentialVarianceAdjuster for ScopeSeasonalCredentialVarianceAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &SeasonalCredentialVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalCredentialVarianceAdjuster {
    pub percent: f64,
}

impl SeasonalCredentialVarianceAdjuster for PercentSeasonalCredentialVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalCredentialVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalCredentialVarianceAdjuster {
    pub minimum: f64,
}

impl SeasonalCredentialVarianceAdjuster for MinimumSeasonalCredentialVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalCredentialVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the seasonal credential variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalCredentialVarianceEngine {
    pub steps: Vec<Rc<dyn SeasonalCredentialVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalCredentialVarianceEngine {
    pub fn compute(&mut self, item: &SeasonalCredentialVariance) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalCredentialVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_credential_variance_engine() -> SeasonalCredentialVarianceEngine {
    SeasonalCredentialVarianceEngine {
        steps: vec![
            Rc::new(ScopeSeasonalCredentialVarianceAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.14), ("retail".to_string(), 0.88)]),
            }) as Rc<dyn SeasonalCredentialVarianceAdjuster>,
            Rc::new(PercentSeasonalCredentialVarianceAdjuster { percent: 5.8 }),
            Rc::new(MinimumSeasonalCredentialVarianceAdjuster { minimum: 36.6 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 46089.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_credential_variance_engine();
        let item = new_seasonal_credential_variance("code-944", "title-857");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
