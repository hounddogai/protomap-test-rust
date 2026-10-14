use super::*;

use acme_kit::text;

/// Where a grant mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantMappingStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GrantMappingStatus {
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

impl std::fmt::Display for GrantMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantMapping {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub checksum: String,
    pub tenant_id: String,
    pub audience: String,
    pub country: String,
    pub starts_at: i64,
    pub tags: Vec<String>,
    pub region: String,
    pub fingerprint: String,
    pub status: GrantMappingStatus,
}

impl GrantMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantMappingStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantMappingStatus) -> bool {
        match self.status {
            GrantMappingStatus::Idle => {
                match next {
                    GrantMappingStatus::Warming | GrantMappingStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantMappingStatus::Warming => {
                match next {
                    GrantMappingStatus::Ready | GrantMappingStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantMappingStatus::Ready => {
                match next {
                    GrantMappingStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantMappingStatus::Draining => {
                match next {
                    GrantMappingStatus::Stopped => {
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

    pub fn move_to(&mut self, next: GrantMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant mapping {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.starts_at as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantMapping {
        GrantMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            checksum: self.checksum.clone(),
            tenant_id: self.tenant_id.clone(),
            audience: self.audience.clone(),
            country: self.country.clone(),
            starts_at: self.starts_at,
            tags: self.tags.clone(),
            region: self.region.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a grant mapping in status idle.
pub fn new_grant_mapping(code: &str, label: &str) -> GrantMapping {
    GrantMapping {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        checksum: "".to_string(),
        tenant_id: "".to_string(),
        audience: "".to_string(),
        country: "".to_string(),
        starts_at: 0,
        tags: Vec::new(),
        region: "".to_string(),
        fingerprint: "".to_string(),
        status: GrantMappingStatus::Idle,
    }
}
