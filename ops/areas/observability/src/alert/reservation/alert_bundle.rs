use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a alert bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AlertBundleStatus {
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

impl std::fmt::Display for AlertBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert bundle as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertBundle {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub channel: String,
    pub threshold: f64,
    pub scope: String,
    pub attributes: BTreeMap<String, String>,
    pub ends_at: i64,
    pub label: String,
    pub pinned: bool,
    pub status: AlertBundleStatus,
}

impl AlertBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertBundleStatus) -> bool {
        match self.status {
            AlertBundleStatus::Idle => {
                match next {
                    AlertBundleStatus::Warming | AlertBundleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertBundleStatus::Warming => {
                match next {
                    AlertBundleStatus::Ready | AlertBundleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertBundleStatus::Ready => {
                match next {
                    AlertBundleStatus::Draining | AlertBundleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertBundleStatus::Draining => {
                return next == AlertBundleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert bundle {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "alert bundle", self.code)
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

    pub fn weighted_total(&self) -> f64 {
        ((self.ends_at as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertBundle {
        AlertBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            channel: self.channel.clone(),
            threshold: self.threshold,
            scope: self.scope.clone(),
            attributes: self.attributes.clone(),
            ends_at: self.ends_at,
            label: self.label.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a alert bundle in status idle.
pub fn new_alert_bundle(code: &str, title: &str) -> AlertBundle {
    AlertBundle {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        channel: "".to_string(),
        threshold: 0.0,
        scope: "".to_string(),
        attributes: BTreeMap::new(),
        ends_at: 0,
        label: "".to_string(),
        pinned: false,
        status: AlertBundleStatus::Idle,
    }
}
