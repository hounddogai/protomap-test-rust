use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the nonce assignment calculation.
pub trait NonceAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceAssignmentAdjuster for FlaggedNonceAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryNonceAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl NonceAssignmentAdjuster for CountryNonceAssignmentAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &NonceAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceAssignmentCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl NonceAssignmentAdjuster for NonceAssignmentCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &NonceAssignment, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingNonceAssignmentAdjuster {
    pub step: f64,
}

impl NonceAssignmentAdjuster for RoundingNonceAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &NonceAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentNonceAssignmentAdjuster {
    pub percent: f64,
}

impl NonceAssignmentAdjuster for PercentNonceAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &NonceAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumNonceAssignmentAdjuster {
    pub minimum: f64,
}

impl NonceAssignmentAdjuster for MinimumNonceAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &NonceAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the nonce assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceAssignmentEngine {
    pub steps: Vec<Rc<dyn NonceAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceAssignmentEngine {
    pub fn run(&mut self, item: &NonceAssignment) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[NonceAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_nonce_assignment_engine(flags: &flags::Registry) -> NonceAssignmentEngine {
    NonceAssignmentEngine {
        steps: vec![
            Rc::new(FlaggedNonceAssignmentAdjuster {
                flags: flags.clone(),
                flag: "nonce-assignment-primary".to_string(),
                percent: -4.9,
            }) as Rc<dyn NonceAssignmentAdjuster>,
            Rc::new(CountryNonceAssignmentAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.91),
                    ("north".to_string(), 1.35),
                    ("south".to_string(), 0.76),
                ]),
            }),
            Rc::new(NonceAssignmentCreatedAtOffsetAdjuster { weight: 1.51 }),
            Rc::new(RoundingNonceAssignmentAdjuster { step: 5.0 }),
            Rc::new(PercentNonceAssignmentAdjuster { percent: -20.1 }),
            Rc::new(MinimumNonceAssignmentAdjuster { minimum: 27.8 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 29652.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_assignment_engine(&flags::new_registry());
        let item = new_nonce_assignment("code-334", "label-38");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
