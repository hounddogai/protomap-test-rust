use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted canary target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedCanaryTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedCanaryTargetStatus {
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

impl std::fmt::Display for WeightedCanaryTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted canary target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCanaryTarget {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub attributes: BTreeMap<String, String>,
    pub grace_days: i64,
    pub name: String,
    pub quota_units: i64,
    pub owner_id: String,
    pub port: i64,
    pub updated_at: i64,
    pub status: WeightedCanaryTargetStatus,
}

impl WeightedCanaryTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedCanaryTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedCanaryTargetStatus) -> bool {
        match self.status {
            WeightedCanaryTargetStatus::Idle => {
                return next == WeightedCanaryTargetStatus::Ready;
            }
            WeightedCanaryTargetStatus::Warming => {
                return next == WeightedCanaryTargetStatus::Ready;
            }
            WeightedCanaryTargetStatus::Ready => {
                return next == WeightedCanaryTargetStatus::Draining || next == WeightedCanaryTargetStatus::Stopped;
            }
            WeightedCanaryTargetStatus::Draining => {
                return next == WeightedCanaryTargetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedCanaryTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted canary target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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

    pub fn effective_value(&self) -> f64 {
        ((self.ceiling + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedCanaryTarget {
        WeightedCanaryTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            attributes: self.attributes.clone(),
            grace_days: self.grace_days,
            name: self.name.clone(),
            quota_units: self.quota_units,
            owner_id: self.owner_id.clone(),
            port: self.port,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted canary target in status idle.
pub fn new_weighted_canary_target(code: &str, title: &str) -> WeightedCanaryTarget {
    WeightedCanaryTarget {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        attributes: BTreeMap::new(),
        grace_days: 0,
        name: "".to_string(),
        quota_units: 0,
        owner_id: "".to_string(),
        port: 0,
        updated_at: 0,
        status: WeightedCanaryTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_canary_target("code-143", "title-224");
        assert_eq!(item.status, WeightedCanaryTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_weighted_canary_target("code-143", "title-224");
        if item.move_to(WeightedCanaryTargetStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedCanaryTargetStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_canary_target("code-143", "title-224");
        item.status = WeightedCanaryTargetStatus::Stopped;
        assert!(
            item.move_to(WeightedCanaryTargetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
