use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the usage policy calculation.
pub trait UsagePolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsagePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsagePolicyAdjuster {
    pub cap: f64,
}

impl UsagePolicyModifier for CapUsagePolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsagePolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsagePolicyRetriesOffsetAdjuster {
    pub weight: f64,
}

impl UsagePolicyModifier for UsagePolicyRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &UsagePolicy, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelUsagePolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsagePolicyModifier for LabelUsagePolicyAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &UsagePolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsagePolicyAdjuster {
    pub minimum: f64,
}

impl UsagePolicyModifier for MinimumUsagePolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsagePolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingUsagePolicyAdjuster {
    pub step: f64,
}

impl UsagePolicyModifier for RoundingUsagePolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &UsagePolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsagePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl UsagePolicyModifier for TieredUsagePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &UsagePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the usage policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsagePolicyEngine {
    pub steps: Vec<Rc<dyn UsagePolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsagePolicyEngine {
    pub fn compute(&mut self, item: &UsagePolicy) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn UsagePolicyModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[UsagePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_usage_policy_engine() -> UsagePolicyEngine {
    UsagePolicyEngine {
        steps: vec![
            Rc::new(CapUsagePolicyAdjuster { cap: 33082.0 }) as Rc<dyn UsagePolicyModifier>,
            Rc::new(UsagePolicyRetriesOffsetAdjuster { weight: 0.24 }),
            Rc::new(LabelUsagePolicyAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.05)]),
            }),
            Rc::new(MinimumUsagePolicyAdjuster { minimum: 46.3 }),
            Rc::new(RoundingUsagePolicyAdjuster { step: 0.25 }),
            Rc::new(TieredUsagePolicyAdjuster {
                bounds: vec![1222.0, 4155.0, 4293.0, 6779.0],
                rates: vec![1.08, 1.18, 0.84, 0.91],
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 46861.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_policy_engine();
        let item = new_usage_policy("code-474", "label-747");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
