use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a health check tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckTierStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl HealthCheckTierStatus {
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

impl std::fmt::Display for HealthCheckTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a health check tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTier {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub region: String,
    pub attributes: BTreeMap<String, String>,
    pub source: String,
    pub ttl_seconds: i64,
    pub key_id: String,
    pub status: HealthCheckTierStatus,
}

impl HealthCheckTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckTierStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckTierStatus) -> bool {
        match self.status {
            HealthCheckTierStatus::New => {
                return next == HealthCheckTierStatus::Running || next == HealthCheckTierStatus::Done;
            }
            HealthCheckTierStatus::Queued => {
                match next {
                    HealthCheckTierStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckTierStatus::Running => {
                return next == HealthCheckTierStatus::Done || next == HealthCheckTierStatus::Failed;
            }
            HealthCheckTierStatus::Done => {
                return next == HealthCheckTierStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check tier {} cannot move from {} to {}",
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

    pub fn projected_load(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckTier {
        HealthCheckTier {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            region: self.region.clone(),
            attributes: self.attributes.clone(),
            source: self.source.clone(),
            ttl_seconds: self.ttl_seconds,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a health check tier in status new.
pub fn new_health_check_tier(code: &str, name: &str) -> HealthCheckTier {
    HealthCheckTier {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        region: "".to_string(),
        attributes: BTreeMap::new(),
        source: "".to_string(),
        ttl_seconds: 0,
        key_id: "".to_string(),
        status: HealthCheckTierStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_health_check_tier("code-516", "name-382");
        assert_eq!(item.status, HealthCheckTierStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_health_check_tier("code-516", "name-382");
        if item.move_to(HealthCheckTierStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckTierStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_health_check_tier("code-516", "name-382");
        item.status = HealthCheckTierStatus::Failed;
        assert!(
            item.move_to(HealthCheckTierStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
