use super::*;

use acme_kit::text;

/// Where a legacy query variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyQueryVarianceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyQueryVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyQueryVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy query variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQueryVariance {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub ceiling: f64,
    pub tags: Vec<String>,
    pub tenant_id: String,
    pub phone: String,
    pub country: String,
    pub floor_value: f64,
    pub rate: f64,
    pub status: LegacyQueryVarianceStatus,
}

impl LegacyQueryVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyQueryVarianceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyQueryVarianceStatus) -> bool {
        match self.status {
            LegacyQueryVarianceStatus::Requested => {
                match next {
                    LegacyQueryVarianceStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyQueryVarianceStatus::Granted => {
                return next == LegacyQueryVarianceStatus::Revoked;
            }
            LegacyQueryVarianceStatus::Revoked => {
                match next {
                    LegacyQueryVarianceStatus::Expired => {
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

    pub fn move_to(&mut self, next: LegacyQueryVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy query variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        (self.floor_value * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyQueryVariance {
        LegacyQueryVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            ceiling: self.ceiling,
            tags: self.tags.clone(),
            tenant_id: self.tenant_id.clone(),
            phone: self.phone.clone(),
            country: self.country.clone(),
            floor_value: self.floor_value,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a legacy query variance in status requested.
pub fn new_legacy_query_variance(code: &str, title: &str) -> LegacyQueryVariance {
    LegacyQueryVariance {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        ceiling: 0.0,
        tags: Vec::new(),
        tenant_id: "".to_string(),
        phone: "".to_string(),
        country: "".to_string(),
        floor_value: 0.0,
        rate: 0.0,
        status: LegacyQueryVarianceStatus::Requested,
    }
}
