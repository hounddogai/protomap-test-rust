use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a adaptive reservation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveReservationProfileStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveReservationProfileStatus {
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

impl std::fmt::Display for AdaptiveReservationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive reservation profile as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReservationProfile {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub ends_at: i64,
    pub revision: i64,
    pub score: f64,
    pub duration_minutes: i64,
    pub discount: f64,
    pub capacity: i64,
    pub attributes: BTreeMap<String, String>,
    pub weight: f64,
    pub status: AdaptiveReservationProfileStatus,
}

impl AdaptiveReservationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveReservationProfileStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveReservationProfileStatus) -> bool {
        match self.status {
            AdaptiveReservationProfileStatus::Idle => {
                return next == AdaptiveReservationProfileStatus::Draining;
            }
            AdaptiveReservationProfileStatus::Warming => {
                return next == AdaptiveReservationProfileStatus::Ready
                    || next == AdaptiveReservationProfileStatus::Draining;
            }
            AdaptiveReservationProfileStatus::Ready => {
                match next {
                    AdaptiveReservationProfileStatus::Draining | AdaptiveReservationProfileStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveReservationProfileStatus::Draining => {
                return next == AdaptiveReservationProfileStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveReservationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive reservation profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive reservation profile", self.code)
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

    pub fn net_volume(&self) -> f64 {
        ((self.weight - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveReservationProfile {
        AdaptiveReservationProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            ends_at: self.ends_at,
            revision: self.revision,
            score: self.score,
            duration_minutes: self.duration_minutes,
            discount: self.discount,
            capacity: self.capacity,
            attributes: self.attributes.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a adaptive reservation profile in status idle.
pub fn new_adaptive_reservation_profile(code: &str, label: &str) -> AdaptiveReservationProfile {
    AdaptiveReservationProfile {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        ends_at: 0,
        revision: 0,
        score: 0.0,
        duration_minutes: 0,
        discount: 0.0,
        capacity: 0,
        attributes: BTreeMap::new(),
        weight: 0.0,
        status: AdaptiveReservationProfileStatus::Idle,
    }
}
