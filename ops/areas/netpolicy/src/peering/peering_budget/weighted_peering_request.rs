use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted peering request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedPeeringRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedPeeringRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for WeightedPeeringRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted peering request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringRequest {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub window_days: i64,
    pub external_ref: String,
    pub ttl_seconds: i64,
    pub updated_at: i64,
    pub port: i64,
    pub title: String,
    pub duration_minutes: i64,
    pub namespace: String,
    pub status: WeightedPeeringRequestStatus,
}

impl WeightedPeeringRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedPeeringRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedPeeringRequestStatus) -> bool {
        match self.status {
            WeightedPeeringRequestStatus::New => {
                return next == WeightedPeeringRequestStatus::Done;
            }
            WeightedPeeringRequestStatus::Queued => {
                return next == WeightedPeeringRequestStatus::Running || next == WeightedPeeringRequestStatus::Done;
            }
            WeightedPeeringRequestStatus::Running => {
                return next == WeightedPeeringRequestStatus::Done || next == WeightedPeeringRequestStatus::Failed;
            }
            WeightedPeeringRequestStatus::Done => {
                return next == WeightedPeeringRequestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedPeeringRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted peering request {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
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

    pub fn net_exposure(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedPeeringRequest {
        WeightedPeeringRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            window_days: self.window_days,
            external_ref: self.external_ref.clone(),
            ttl_seconds: self.ttl_seconds,
            updated_at: self.updated_at,
            port: self.port,
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a weighted peering request in status new.
pub fn new_weighted_peering_request(code: &str, name: &str) -> WeightedPeeringRequest {
    WeightedPeeringRequest {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        window_days: 0,
        external_ref: "".to_string(),
        ttl_seconds: 0,
        updated_at: 0,
        port: 0,
        title: "".to_string(),
        duration_minutes: 0,
        namespace: "".to_string(),
        status: WeightedPeeringRequestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_weighted_peering_request("code-522", "name-238");
        assert_eq!(item.status, WeightedPeeringRequestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_weighted_peering_request("code-522", "name-238");
        if item.move_to(WeightedPeeringRequestStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedPeeringRequestStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_weighted_peering_request("code-522", "name-238");
        item.status = WeightedPeeringRequestStatus::Failed;
        assert!(
            item.move_to(WeightedPeeringRequestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
