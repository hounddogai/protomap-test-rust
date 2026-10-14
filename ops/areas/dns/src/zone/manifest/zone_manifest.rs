use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a zone manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ZoneManifestStatus {
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

impl std::fmt::Display for ZoneManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One zone manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneManifest {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub sequence: i64,
    pub budget: f64,
    pub duration_minutes: i64,
    pub region: String,
    pub unit: String,
    pub scope: String,
    pub burst: i64,
    pub pinned: bool,
    pub status: ZoneManifestStatus,
}

impl ZoneManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneManifestStatus) -> bool {
        match self.status {
            ZoneManifestStatus::Idle => {
                match next {
                    ZoneManifestStatus::Warming | ZoneManifestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneManifestStatus::Warming => {
                return next == ZoneManifestStatus::Ready;
            }
            ZoneManifestStatus::Ready => {
                match next {
                    ZoneManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneManifestStatus::Draining => {
                return next == ZoneManifestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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
        ((self.duration_minutes as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneManifest {
        ZoneManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            sequence: self.sequence,
            budget: self.budget,
            duration_minutes: self.duration_minutes,
            region: self.region.clone(),
            unit: self.unit.clone(),
            scope: self.scope.clone(),
            burst: self.burst,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a zone manifest in status idle.
pub fn new_zone_manifest(code: &str, title: &str) -> ZoneManifest {
    ZoneManifest {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        sequence: 0,
        budget: 0.0,
        duration_minutes: 0,
        region: "".to_string(),
        unit: "".to_string(),
        scope: "".to_string(),
        burst: 0,
        pinned: false,
        status: ZoneManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_zone_manifest("code-750", "title-631");
        assert_eq!(item.status, ZoneManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_zone_manifest("code-750", "title-631");
        if item.move_to(ZoneManifestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneManifestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_zone_manifest("code-750", "title-631");
        item.status = ZoneManifestStatus::Stopped;
        assert!(item.move_to(ZoneManifestStatus::Idle).is_err(), "expected move to fail");
    }
}
