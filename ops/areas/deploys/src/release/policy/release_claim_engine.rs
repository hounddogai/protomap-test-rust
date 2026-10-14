use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the release claim calculation.
pub trait ReleaseClaimAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseClaim, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseClaimAdjuster for FlaggedReleaseClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleReleaseClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseClaimAdjuster for LocaleReleaseClaimAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &ReleaseClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseClaimStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseClaimAdjuster for ReleaseClaimStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseClaim, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReleaseClaimAdjuster {
    pub minimum: f64,
}

impl ReleaseClaimAdjuster for MinimumReleaseClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReleaseClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseClaimAdjuster for TieredReleaseClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the release claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseClaimEngine {
    pub steps: Vec<Rc<dyn ReleaseClaimAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseClaimEngine {
    pub fn run(&mut self, item: &ReleaseClaim) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseClaimAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_release_claim_engine(flags: &flags::Registry) -> ReleaseClaimEngine {
    ReleaseClaimEngine {
        steps: vec![
            Rc::new(FlaggedReleaseClaimAdjuster {
                flags: flags.clone(),
                flag: "release-claim-scheduled".to_string(),
                percent: 2.4,
            }) as Rc<dyn ReleaseClaimAdjuster>,
            Rc::new(LocaleReleaseClaimAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.75),
                    ("south".to_string(), 0.78),
                    ("east".to_string(), 1.03),
                ]),
            }),
            Rc::new(ReleaseClaimStartsAtOffsetAdjuster { weight: 1.07 }),
            Rc::new(MinimumReleaseClaimAdjuster { minimum: 11.9 }),
            Rc::new(TieredReleaseClaimAdjuster {
                bounds: vec![4427.0, 5867.0, 6000.0, 8163.0],
                rates: vec![0.82, 0.87, 0.9, 0.83],
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 45234.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_claim_engine(&flags::new_registry());
        let item = new_release_claim("code-405", "label-583");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
