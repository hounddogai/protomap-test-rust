use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a region pin digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinDigestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionPinDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RegionPinDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinDigest {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub attributes: BTreeMap<String, String>,
    pub external_ref: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub duration_minutes: i64,
    pub threshold: f64,
    pub starts_at: i64,
    pub unit: String,
    pub status: RegionPinDigestStatus,
}

impl RegionPinDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinDigestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinDigestStatus) -> bool {
        match self.status {
            RegionPinDigestStatus::Draft => {
                match next {
                    RegionPinDigestStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinDigestStatus::Pending => {
                match next {
                    RegionPinDigestStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinDigestStatus::Active => {
                return next == RegionPinDigestStatus::Suspended || next == RegionPinDigestStatus::Closed;
            }
            RegionPinDigestStatus::Suspended => {
                return next == RegionPinDigestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn effective_total(&self) -> f64 {
        ((self.created_at as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinDigest {
        RegionPinDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            attributes: self.attributes.clone(),
            external_ref: self.external_ref.clone(),
            created_at: self.created_at,
            expires_at: self.expires_at,
            duration_minutes: self.duration_minutes,
            threshold: self.threshold,
            starts_at: self.starts_at,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a region pin digest in status draft.
pub fn new_region_pin_digest(code: &str, title: &str) -> RegionPinDigest {
    RegionPinDigest {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        attributes: BTreeMap::new(),
        external_ref: "".to_string(),
        created_at: 0,
        expires_at: 0,
        duration_minutes: 0,
        threshold: 0.0,
        starts_at: 0,
        unit: "".to_string(),
        status: RegionPinDigestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_region_pin_digest("code-205", "title-959");
        assert_eq!(item.status, RegionPinDigestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_region_pin_digest("code-205", "title-959");
        if item.move_to(RegionPinDigestStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinDigestStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_region_pin_digest("code-205", "title-959");
        item.status = RegionPinDigestStatus::Closed;
        assert!(
            item.move_to(RegionPinDigestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
