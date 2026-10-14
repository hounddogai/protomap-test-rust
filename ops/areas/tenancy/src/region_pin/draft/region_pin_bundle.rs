use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a region pin bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinBundleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionPinBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionPinBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBundle {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub title: String,
    pub checksum: String,
    pub starts_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub category: String,
    pub ttl_seconds: i64,
    pub grace_days: i64,
    pub description: String,
    pub status: RegionPinBundleStatus,
}

impl RegionPinBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinBundleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinBundleStatus) -> bool {
        match self.status {
            RegionPinBundleStatus::Created => {
                return next == RegionPinBundleStatus::Published;
            }
            RegionPinBundleStatus::Validated => {
                return next == RegionPinBundleStatus::Published || next == RegionPinBundleStatus::Retired;
            }
            RegionPinBundleStatus::Published => {
                match next {
                    RegionPinBundleStatus::Retired => {
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

    pub fn move_to(&mut self, next: RegionPinBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "region pin bundle", self.code)
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

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.grace_days as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinBundle {
        RegionPinBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            starts_at: self.starts_at,
            attributes: self.attributes.clone(),
            category: self.category.clone(),
            ttl_seconds: self.ttl_seconds,
            grace_days: self.grace_days,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a region pin bundle in status created.
pub fn new_region_pin_bundle(code: &str, name: &str) -> RegionPinBundle {
    RegionPinBundle {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        title: "".to_string(),
        checksum: "".to_string(),
        starts_at: 0,
        attributes: BTreeMap::new(),
        category: "".to_string(),
        ttl_seconds: 0,
        grace_days: 0,
        description: "".to_string(),
        status: RegionPinBundleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_region_pin_bundle("code-377", "name-123");
        assert_eq!(item.status, RegionPinBundleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_region_pin_bundle("code-377", "name-123");
        if item.move_to(RegionPinBundleStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinBundleStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_region_pin_bundle("code-377", "name-123");
        item.status = RegionPinBundleStatus::Retired;
        assert!(
            item.move_to(RegionPinBundleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
