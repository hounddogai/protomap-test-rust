use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval manifest calculation.
pub trait ApprovalManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalManifestAdjuster {
    pub step: f64,
}

impl ApprovalManifestModifier for RoundingApprovalManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalManifestFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalManifestModifier for ApprovalManifestFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalManifest, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalManifestAdjuster {
    pub percent: f64,
}

impl ApprovalManifestModifier for PercentApprovalManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameApprovalManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalManifestModifier for NameApprovalManifestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ApprovalManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalManifestModifier for TieredApprovalManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the approval manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalManifestEngine {
    pub steps: Vec<Rc<dyn ApprovalManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalManifestEngine {
    pub fn run(&mut self, item: &ApprovalManifest) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_approval_manifest_engine() -> ApprovalManifestEngine {
    ApprovalManifestEngine {
        steps: vec![
            Rc::new(RoundingApprovalManifestAdjuster { step: 5.0 }) as Rc<dyn ApprovalManifestModifier>,
            Rc::new(ApprovalManifestFloorValueOffsetAdjuster { weight: 1.26 }),
            Rc::new(PercentApprovalManifestAdjuster { percent: -23.4 }),
            Rc::new(NameApprovalManifestAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.24),
                    ("south".to_string(), 0.93),
                    ("central".to_string(), 0.72),
                ]),
            }),
            Rc::new(TieredApprovalManifestAdjuster {
                bounds: vec![3629.0, 4628.0, 7861.0, 8973.0],
                rates: vec![1.18, 1.16, 0.82, 0.87],
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 31778.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_manifest_engine();
        let item = new_approval_manifest("code-869", "name-732");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
