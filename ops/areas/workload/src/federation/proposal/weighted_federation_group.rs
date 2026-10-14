use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted federation group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFederationGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedFederationGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for WeightedFederationGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted federation group as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationGroup {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub pinned: bool,
    pub region: String,
    pub weight: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: WeightedFederationGroupStatus,
}

impl WeightedFederationGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFederationGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFederationGroupStatus) -> bool {
        match self.status {
            WeightedFederationGroupStatus::Idle => {
                match next {
                    WeightedFederationGroupStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFederationGroupStatus::Warming => {
                return next == WeightedFederationGroupStatus::Ready;
            }
            WeightedFederationGroupStatus::Ready => {
                match next {
                    WeightedFederationGroupStatus::Draining | WeightedFederationGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFederationGroupStatus::Draining => {
                return next == WeightedFederationGroupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedFederationGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted federation group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_value(&self) -> f64 {
        ((self.window_days as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedFederationGroup {
        WeightedFederationGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            pinned: self.pinned,
            region: self.region.clone(),
            weight: self.weight,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a weighted federation group in status idle.
pub fn new_weighted_federation_group(code: &str, name: &str) -> WeightedFederationGroup {
    WeightedFederationGroup {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        pinned: false,
        region: "".to_string(),
        weight: 0.0,
        attributes: BTreeMap::new(),
        status: WeightedFederationGroupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_federation_group("code-167", "name-947");
        assert_eq!(item.status, WeightedFederationGroupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_weighted_federation_group("code-167", "name-947");
        if item.move_to(WeightedFederationGroupStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedFederationGroupStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_federation_group("code-167", "name-947");
        item.status = WeightedFederationGroupStatus::Stopped;
        assert!(
            item.move_to(WeightedFederationGroupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
