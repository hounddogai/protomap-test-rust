use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient zone preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientZonePreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientZonePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientZonePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient zone preference as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientZonePreference {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub starts_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub source: String,
    pub ratio: f64,
    pub external_ref: String,
    pub status: LenientZonePreferenceStatus,
}

impl LenientZonePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientZonePreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientZonePreferenceStatus) -> bool {
        match self.status {
            LenientZonePreferenceStatus::Created => {
                match next {
                    LenientZonePreferenceStatus::Validated | LenientZonePreferenceStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientZonePreferenceStatus::Validated => {
                match next {
                    LenientZonePreferenceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientZonePreferenceStatus::Published => {
                match next {
                    LenientZonePreferenceStatus::Retired => {
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

    pub fn move_to(&mut self, next: LenientZonePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient zone preference {} cannot move from {} to {}",
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.floor_value - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientZonePreference {
        LenientZonePreference {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            starts_at: self.starts_at,
            attributes: self.attributes.clone(),
            source: self.source.clone(),
            ratio: self.ratio,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a lenient zone preference in status created.
pub fn new_lenient_zone_preference(code: &str, title: &str) -> LenientZonePreference {
    LenientZonePreference {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        starts_at: 0,
        attributes: BTreeMap::new(),
        source: "".to_string(),
        ratio: 0.0,
        external_ref: "".to_string(),
        status: LenientZonePreferenceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_zone_preference("code-740", "title-378");
        assert_eq!(item.status, LenientZonePreferenceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_zone_preference("code-740", "title-378");
        if item.move_to(LenientZonePreferenceStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientZonePreferenceStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_zone_preference("code-740", "title-378");
        item.status = LenientZonePreferenceStatus::Retired;
        assert!(
            item.move_to(LenientZonePreferenceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
