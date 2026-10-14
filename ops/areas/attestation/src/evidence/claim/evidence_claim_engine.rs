use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the evidence claim calculation.
pub trait EvidenceClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceClaimAdjuster {
    pub minimum: f64,
}

impl EvidenceClaimFactor for MinimumEvidenceClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameEvidenceClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceClaimFactor for NameEvidenceClaimAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &EvidenceClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceClaimCeilingOffsetAdjuster {
    pub weight: f64,
}

impl EvidenceClaimFactor for EvidenceClaimCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &EvidenceClaim, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceClaimFactor for TieredEvidenceClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the evidence claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceClaimEngine {
    pub steps: Vec<Rc<dyn EvidenceClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceClaimEngine {
    pub fn price(&mut self, item: &EvidenceClaim) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_evidence_claim_engine() -> EvidenceClaimEngine {
    EvidenceClaimEngine {
        steps: vec![
            Rc::new(MinimumEvidenceClaimAdjuster { minimum: 31.9 }) as Rc<dyn EvidenceClaimFactor>,
            Rc::new(NameEvidenceClaimAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.89),
                    ("north".to_string(), 1.22),
                    ("retail".to_string(), 1.34),
                ]),
            }),
            Rc::new(EvidenceClaimCeilingOffsetAdjuster { weight: 1.93 }),
            Rc::new(TieredEvidenceClaimAdjuster {
                bounds: vec![3340.0, 4308.0, 7831.0, 9223.0],
                rates: vec![1.0, 1.19, 0.89, 1.18],
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 86568.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_claim_engine();
        let item = new_evidence_claim("code-796", "name-122");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
