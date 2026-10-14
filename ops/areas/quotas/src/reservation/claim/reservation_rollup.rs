use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a reservation rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationRollupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReservationRollupStatus {
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

impl std::fmt::Display for ReservationRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationRollup {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub contact_email: String,
    pub principal: String,
    pub threshold: f64,
    pub ttl_seconds: i64,
    pub audience: String,
    pub attributes: BTreeMap<String, String>,
    pub status: ReservationRollupStatus,
}

impl ReservationRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationRollupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationRollupStatus) -> bool {
        match self.status {
            ReservationRollupStatus::Idle => {
                return next == ReservationRollupStatus::Warming || next == ReservationRollupStatus::Draining;
            }
            ReservationRollupStatus::Warming => {
                match next {
                    ReservationRollupStatus::Ready | ReservationRollupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationRollupStatus::Ready => {
                match next {
                    ReservationRollupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationRollupStatus::Draining => {
                return next == ReservationRollupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn adjusted_total(&self) -> f64 {
        ((self.threshold - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationRollup {
        ReservationRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            contact_email: self.contact_email.clone(),
            principal: self.principal.clone(),
            threshold: self.threshold,
            ttl_seconds: self.ttl_seconds,
            audience: self.audience.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a reservation rollup in status idle.
pub fn new_reservation_rollup(code: &str, label: &str) -> ReservationRollup {
    ReservationRollup {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        contact_email: "".to_string(),
        principal: "".to_string(),
        threshold: 0.0,
        ttl_seconds: 0,
        audience: "".to_string(),
        attributes: BTreeMap::new(),
        status: ReservationRollupStatus::Idle,
    }
}
