use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted budget burn audit calculation.
pub trait WeightedBudgetBurnAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedBudgetBurnAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedBudgetBurnAuditAdjuster {
    pub step: f64,
}

impl WeightedBudgetBurnAuditFactor for RoundingWeightedBudgetBurnAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedBudgetBurnAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnAuditExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl WeightedBudgetBurnAuditFactor for WeightedBudgetBurnAuditExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedBudgetBurnAudit, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedBudgetBurnAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedBudgetBurnAuditFactor for TieredWeightedBudgetBurnAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedBudgetBurnAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedBudgetBurnAuditAdjuster {
    pub percent: f64,
}

impl WeightedBudgetBurnAuditFactor for PercentWeightedBudgetBurnAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedBudgetBurnAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedBudgetBurnAuditAdjuster {
    pub cap: f64,
}

impl WeightedBudgetBurnAuditFactor for CapWeightedBudgetBurnAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedBudgetBurnAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameWeightedBudgetBurnAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedBudgetBurnAuditFactor for NameWeightedBudgetBurnAuditAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &WeightedBudgetBurnAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted budget burn audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedBudgetBurnAuditEngine {
    pub steps: Vec<Rc<dyn WeightedBudgetBurnAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedBudgetBurnAuditEngine {
    pub fn evaluate(&mut self, item: &WeightedBudgetBurnAudit) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[WeightedBudgetBurnAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_budget_burn_audit_engine() -> WeightedBudgetBurnAuditEngine {
    WeightedBudgetBurnAuditEngine {
        steps: vec![
            Rc::new(RoundingWeightedBudgetBurnAuditAdjuster { step: 0.05 }) as Rc<dyn WeightedBudgetBurnAuditFactor>,
            Rc::new(WeightedBudgetBurnAuditExpiresAtOffsetAdjuster { weight: 0.89 }),
            Rc::new(TieredWeightedBudgetBurnAuditAdjuster {
                bounds: vec![2760.0, 3328.0, 9586.0, 9586.0],
                rates: vec![0.82, 1.0, 0.87, 0.83],
            }),
            Rc::new(PercentWeightedBudgetBurnAuditAdjuster { percent: 4.0 }),
            Rc::new(CapWeightedBudgetBurnAuditAdjuster { cap: 48892.0 }),
            Rc::new(NameWeightedBudgetBurnAuditAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.37)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 36675.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_budget_burn_audit_engine();
        let item = new_weighted_budget_burn_audit("code-59", "name-65");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
