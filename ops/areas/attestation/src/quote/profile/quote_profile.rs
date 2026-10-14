use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a quote profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteProfileStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QuoteProfileStatus {
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

impl std::fmt::Display for QuoteProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteProfile {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub version: i64,
    pub country: String,
    pub burst: i64,
    pub attributes: BTreeMap<String, String>,
    pub revision: i64,
    pub attempts: i64,
    pub status: QuoteProfileStatus,
}

impl QuoteProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteProfileStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteProfileStatus) -> bool {
        match self.status {
            QuoteProfileStatus::Idle => {
                match next {
                    QuoteProfileStatus::Draining | QuoteProfileStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteProfileStatus::Warming => {
                match next {
                    QuoteProfileStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteProfileStatus::Ready => {
                return next == QuoteProfileStatus::Draining;
            }
            QuoteProfileStatus::Draining => {
                return next == QuoteProfileStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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

    pub fn projected_value(&self) -> f64 {
        ((self.burst as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteProfile {
        QuoteProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            version: self.version,
            country: self.country.clone(),
            burst: self.burst,
            attributes: self.attributes.clone(),
            revision: self.revision,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a quote profile in status idle.
pub fn new_quote_profile(code: &str, name: &str) -> QuoteProfile {
    QuoteProfile {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        version: 0,
        country: "".to_string(),
        burst: 0,
        attributes: BTreeMap::new(),
        revision: 0,
        attempts: 0,
        status: QuoteProfileStatus::Idle,
    }
}
