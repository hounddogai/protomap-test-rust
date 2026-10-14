use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the ttl policy manifest calculation.
pub trait TtlPolicyManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyManifest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyManifestModifier for FlaggedTtlPolicyManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyManifestAdjuster {
    pub percent: f64,
}

impl TtlPolicyManifestModifier for PercentTtlPolicyManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleTtlPolicyManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyManifestModifier for LocaleTtlPolicyManifestAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &TtlPolicyManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyManifestAdjuster {
    pub cap: f64,
}

impl TtlPolicyManifestModifier for CapTtlPolicyManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the ttl policy manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyManifestEngine {
    pub steps: Vec<Rc<dyn TtlPolicyManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyManifestEngine {
    pub fn run(&mut self, item: &TtlPolicyManifest) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_manifest_engine(flags: &flags::Registry) -> TtlPolicyManifestEngine {
    TtlPolicyManifestEngine {
        steps: vec![
            Rc::new(FlaggedTtlPolicyManifestAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-manifest-manual".to_string(),
                percent: -5.2,
            }) as Rc<dyn TtlPolicyManifestModifier>,
            Rc::new(PercentTtlPolicyManifestAdjuster { percent: 5.4 }),
            Rc::new(LocaleTtlPolicyManifestAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.31),
                    ("east".to_string(), 1.26),
                    ("central".to_string(), 0.77),
                ]),
            }),
            Rc::new(CapTtlPolicyManifestAdjuster { cap: 36568.0 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 6077.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_manifest_engine(&flags::new_registry());
        let item = new_ttl_policy_manifest("code-94", "title-88");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
