use super::*;

use acme_kit::text;

/// Where a zone profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneProfileStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ZoneProfileStatus {
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

impl std::fmt::Display for ZoneProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone profile as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneProfile {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub batch_size: i64,
    pub ceiling: f64,
    pub source: String,
    pub tags: Vec<String>,
    pub active: bool,
    pub priority: i64,
    pub status: ZoneProfileStatus,
}

impl ZoneProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneProfileStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneProfileStatus) -> bool {
        match self.status {
            ZoneProfileStatus::Idle => {
                return next == ZoneProfileStatus::Warming;
            }
            ZoneProfileStatus::Warming => {
                return next == ZoneProfileStatus::Draining;
            }
            ZoneProfileStatus::Ready => {
                match next {
                    ZoneProfileStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneProfileStatus::Draining => {
                match next {
                    ZoneProfileStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ZoneProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "zone profile", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.priority as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneProfile {
        ZoneProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            batch_size: self.batch_size,
            ceiling: self.ceiling,
            source: self.source.clone(),
            tags: self.tags.clone(),
            active: self.active,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a zone profile in status idle.
pub fn new_zone_profile(code: &str, name: &str) -> ZoneProfile {
    ZoneProfile {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        batch_size: 0,
        ceiling: 0.0,
        source: "".to_string(),
        tags: Vec::new(),
        active: false,
        priority: 0,
        status: ZoneProfileStatus::Idle,
    }
}
