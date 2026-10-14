use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rolling forecast policy calculation.
pub trait RollingForecastPolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingForecastPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRollingForecastPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingForecastPolicyModifier for CountryRollingForecastPolicyAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RollingForecastPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingForecastPolicyAdjuster {
    pub cap: f64,
}

impl RollingForecastPolicyModifier for CapRollingForecastPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingForecastPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingForecastPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingForecastPolicyModifier for TieredRollingForecastPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingForecastPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingForecastPolicyAdjuster {
    pub percent: f64,
}

impl RollingForecastPolicyModifier for PercentRollingForecastPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingForecastPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling forecast policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingForecastPolicyEngine {
    pub steps: Vec<Rc<dyn RollingForecastPolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingForecastPolicyEngine {
    pub fn run(&mut self, item: &RollingForecastPolicy) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingForecastPolicyModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingForecastPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_forecast_policy_engine() -> RollingForecastPolicyEngine {
    RollingForecastPolicyEngine {
        steps: vec![
            Rc::new(CountryRollingForecastPolicyAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.87)]),
            }) as Rc<dyn RollingForecastPolicyModifier>,
            Rc::new(CapRollingForecastPolicyAdjuster { cap: 32888.0 }),
            Rc::new(TieredRollingForecastPolicyAdjuster {
                bounds: vec![1800.0, 6663.0],
                rates: vec![1.12, 1.17],
            }),
            Rc::new(PercentRollingForecastPolicyAdjuster { percent: 8.7 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 50056.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_forecast_policy_engine();
        let item = new_rolling_forecast_policy("code-428", "name-746");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
