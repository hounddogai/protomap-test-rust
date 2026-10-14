use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a tiered query manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredQueryManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredQueryManifestStatus {
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

impl std::fmt::Display for TieredQueryManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered query manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryManifest {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub grace_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub floor_value: f64,
    pub unit: String,
    pub threshold: f64,
    pub status: TieredQueryManifestStatus,
}

impl TieredQueryManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredQueryManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredQueryManifestStatus) -> bool {
        match self.status {
            TieredQueryManifestStatus::Idle => {
                match next {
                    TieredQueryManifestStatus::Warming | TieredQueryManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredQueryManifestStatus::Warming => {
                match next {
                    TieredQueryManifestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredQueryManifestStatus::Ready => {
                match next {
                    TieredQueryManifestStatus::Draining | TieredQueryManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredQueryManifestStatus::Draining => {
                return next == TieredQueryManifestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredQueryManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered query manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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

    pub fn weighted_exposure(&self) -> f64 {
        ((self.floor_value + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredQueryManifest {
        TieredQueryManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            grace_days: self.grace_days,
            attributes: self.attributes.clone(),
            floor_value: self.floor_value,
            unit: self.unit.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tiered query manifest in status idle.
pub fn new_tiered_query_manifest(code: &str, title: &str) -> TieredQueryManifest {
    TieredQueryManifest {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        grace_days: 0,
        attributes: BTreeMap::new(),
        floor_value: 0.0,
        unit: "".to_string(),
        threshold: 0.0,
        status: TieredQueryManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_query_manifest("code-659", "title-183");
        assert_eq!(item.status, TieredQueryManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_tiered_query_manifest("code-659", "title-183");
        if item.move_to(TieredQueryManifestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredQueryManifestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_query_manifest("code-659", "title-183");
        item.status = TieredQueryManifestStatus::Stopped;
        assert!(
            item.move_to(TieredQueryManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
