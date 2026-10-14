use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a adaptive rollout reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRolloutReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveRolloutReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for AdaptiveRolloutReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive rollout reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutReservation {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub segment: String,
    pub attributes: BTreeMap<String, String>,
    pub host: String,
    pub country: String,
    pub ttl_seconds: i64,
    pub ends_at: i64,
    pub tags: Vec<String>,
    pub retries: i64,
    pub status: AdaptiveRolloutReservationStatus,
}

impl AdaptiveRolloutReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRolloutReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRolloutReservationStatus) -> bool {
        match self.status {
            AdaptiveRolloutReservationStatus::Open => {
                return next == AdaptiveRolloutReservationStatus::Archived;
            }
            AdaptiveRolloutReservationStatus::InReview => {
                return next == AdaptiveRolloutReservationStatus::Archived;
            }
            AdaptiveRolloutReservationStatus::Resolved => {
                return next == AdaptiveRolloutReservationStatus::Reopened;
            }
            AdaptiveRolloutReservationStatus::Reopened => {
                match next {
                    AdaptiveRolloutReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRolloutReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive rollout reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn projected_exposure(&self) -> f64 {
        ((self.port as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRolloutReservation {
        AdaptiveRolloutReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            segment: self.segment.clone(),
            attributes: self.attributes.clone(),
            host: self.host.clone(),
            country: self.country.clone(),
            ttl_seconds: self.ttl_seconds,
            ends_at: self.ends_at,
            tags: self.tags.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a adaptive rollout reservation in status open.
pub fn new_adaptive_rollout_reservation(code: &str, title: &str) -> AdaptiveRolloutReservation {
    AdaptiveRolloutReservation {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        segment: "".to_string(),
        attributes: BTreeMap::new(),
        host: "".to_string(),
        country: "".to_string(),
        ttl_seconds: 0,
        ends_at: 0,
        tags: Vec::new(),
        retries: 0,
        status: AdaptiveRolloutReservationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_rollout_reservation("code-8", "title-257");
        assert_eq!(item.status, AdaptiveRolloutReservationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_adaptive_rollout_reservation("code-8", "title-257");
        if item.move_to(AdaptiveRolloutReservationStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRolloutReservationStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_rollout_reservation("code-8", "title-257");
        item.status = AdaptiveRolloutReservationStatus::Archived;
        assert!(
            item.move_to(AdaptiveRolloutReservationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
