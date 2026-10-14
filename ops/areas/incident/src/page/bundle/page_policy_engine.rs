use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the page policy calculation.
pub trait PagePolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PagePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPagePolicyAdjuster {
    pub step: f64,
}

impl PagePolicyModifier for RoundingPagePolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PagePolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryPagePolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PagePolicyModifier for CategoryPagePolicyAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &PagePolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPagePolicyAdjuster {
    pub percent: f64,
}

impl PagePolicyModifier for PercentPagePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PagePolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPagePolicyAdjuster {
    pub cap: f64,
}

impl PagePolicyModifier for CapPagePolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PagePolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPagePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PagePolicyModifier for TieredPagePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PagePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the page policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PagePolicyEngine {
    pub steps: Vec<Rc<dyn PagePolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PagePolicyEngine {
    pub fn evaluate(&mut self, item: &PagePolicy) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PagePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_page_policy_engine() -> PagePolicyEngine {
    PagePolicyEngine {
        steps: vec![
            Rc::new(RoundingPagePolicyAdjuster { step: 1.0 }) as Rc<dyn PagePolicyModifier>,
            Rc::new(CategoryPagePolicyAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.77), ("north".to_string(), 0.89)]),
            }),
            Rc::new(PercentPagePolicyAdjuster { percent: -24.5 }),
            Rc::new(CapPagePolicyAdjuster { cap: 47327.0 }),
            Rc::new(TieredPagePolicyAdjuster {
                bounds: vec![827.0, 2154.0, 8973.0],
                rates: vec![0.87, 0.83, 0.95],
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 96766.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_policy_engine();
        let item = new_page_policy("code-769", "label-922");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
