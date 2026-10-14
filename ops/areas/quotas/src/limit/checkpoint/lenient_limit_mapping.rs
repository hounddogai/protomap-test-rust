use super::*;

use acme_kit::text;

/// Where a lenient limit mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientLimitMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientLimitMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientLimitMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient limit mapping as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitMapping {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub tags: Vec<String>,
    pub region: String,
    pub segment: String,
    pub cipher: String,
    pub cost: f64,
    pub country: String,
    pub status: LenientLimitMappingStatus,
}

impl LenientLimitMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientLimitMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientLimitMappingStatus) -> bool {
        match self.status {
            LenientLimitMappingStatus::Requested => {
                match next {
                    LenientLimitMappingStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientLimitMappingStatus::Granted => {
                match next {
                    LenientLimitMappingStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientLimitMappingStatus::Revoked => {
                return next == LenientLimitMappingStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientLimitMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient limit mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.cost + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientLimitMapping {
        LenientLimitMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            tags: self.tags.clone(),
            region: self.region.clone(),
            segment: self.segment.clone(),
            cipher: self.cipher.clone(),
            cost: self.cost,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a lenient limit mapping in status requested.
pub fn new_lenient_limit_mapping(code: &str, title: &str) -> LenientLimitMapping {
    LenientLimitMapping {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        tags: Vec::new(),
        region: "".to_string(),
        segment: "".to_string(),
        cipher: "".to_string(),
        cost: 0.0,
        country: "".to_string(),
        status: LenientLimitMappingStatus::Requested,
    }
}
