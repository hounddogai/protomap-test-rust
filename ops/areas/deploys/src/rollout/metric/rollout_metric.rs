use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rollout metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RolloutMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RolloutMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMetric {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub attributes: BTreeMap<String, String>,
    pub currency: String,
    pub notes: String,
    pub key_id: String,
    pub ttl_seconds: i64,
    pub owner_id: String,
    pub status: RolloutMetricStatus,
}

impl RolloutMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutMetricStatus) -> bool {
        match self.status {
            RolloutMetricStatus::Created => {
                match next {
                    RolloutMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutMetricStatus::Validated => {
                match next {
                    RolloutMetricStatus::Published | RolloutMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutMetricStatus::Published => {
                return next == RolloutMetricStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollout metric", self.code)
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

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> RolloutMetric {
        RolloutMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            attributes: self.attributes.clone(),
            currency: self.currency.clone(),
            notes: self.notes.clone(),
            key_id: self.key_id.clone(),
            ttl_seconds: self.ttl_seconds,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rollout metric in status created.
pub fn new_rollout_metric(code: &str, title: &str) -> RolloutMetric {
    RolloutMetric {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        attributes: BTreeMap::new(),
        currency: "".to_string(),
        notes: "".to_string(),
        key_id: "".to_string(),
        ttl_seconds: 0,
        owner_id: "".to_string(),
        status: RolloutMetricStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rollout_metric("code-520", "title-98");
        assert_eq!(item.status, RolloutMetricStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_rollout_metric("code-520", "title-98");
        if item.move_to(RolloutMetricStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutMetricStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rollout_metric("code-520", "title-98");
        item.status = RolloutMetricStatus::Retired;
        assert!(
            item.move_to(RolloutMetricStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
