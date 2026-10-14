use super::*;

use acme_kit::text;

/// Where a fallback isolation record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackIsolationRecordStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackIsolationRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackIsolationRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback isolation record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationRecord {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub tags: Vec<String>,
    pub ttl_seconds: i64,
    pub minimum: f64,
    pub budget: f64,
    pub principal: String,
    pub score: f64,
    pub rate: f64,
    pub status: FallbackIsolationRecordStatus,
}

impl FallbackIsolationRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackIsolationRecordStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackIsolationRecordStatus) -> bool {
        match self.status {
            FallbackIsolationRecordStatus::Requested => {
                match next {
                    FallbackIsolationRecordStatus::Granted | FallbackIsolationRecordStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackIsolationRecordStatus::Granted => {
                return next == FallbackIsolationRecordStatus::Revoked;
            }
            FallbackIsolationRecordStatus::Revoked => {
                match next {
                    FallbackIsolationRecordStatus::Expired => {
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

    pub fn move_to(&mut self, next: FallbackIsolationRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback isolation record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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
        ((self.rate - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackIsolationRecord {
        FallbackIsolationRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            tags: self.tags.clone(),
            ttl_seconds: self.ttl_seconds,
            minimum: self.minimum,
            budget: self.budget,
            principal: self.principal.clone(),
            score: self.score,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a fallback isolation record in status requested.
pub fn new_fallback_isolation_record(code: &str, title: &str) -> FallbackIsolationRecord {
    FallbackIsolationRecord {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        tags: Vec::new(),
        ttl_seconds: 0,
        minimum: 0.0,
        budget: 0.0,
        principal: "".to_string(),
        score: 0.0,
        rate: 0.0,
        status: FallbackIsolationRecordStatus::Requested,
    }
}
