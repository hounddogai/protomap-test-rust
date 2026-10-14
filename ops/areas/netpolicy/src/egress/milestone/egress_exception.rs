use super::*;

use acme_kit::text;

/// Where a egress exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EgressExceptionStatus {
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

impl std::fmt::Display for EgressExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressException {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub checksum: String,
    pub burst: i64,
    pub rotation_days: i64,
    pub phone: String,
    pub attempts: i64,
    pub tags: Vec<String>,
    pub notes: String,
    pub unit: String,
    pub status: EgressExceptionStatus,
}

impl EgressException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressExceptionStatus) -> bool {
        match self.status {
            EgressExceptionStatus::Draft => {
                return next == EgressExceptionStatus::Pending || next == EgressExceptionStatus::Active;
            }
            EgressExceptionStatus::Pending => {
                match next {
                    EgressExceptionStatus::Active | EgressExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressExceptionStatus::Active => {
                match next {
                    EgressExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressExceptionStatus::Suspended => {
                return next == EgressExceptionStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress exception {} cannot move from {} to {}",
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
        ((self.attempts as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressException {
        EgressException {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            checksum: self.checksum.clone(),
            burst: self.burst,
            rotation_days: self.rotation_days,
            phone: self.phone.clone(),
            attempts: self.attempts,
            tags: self.tags.clone(),
            notes: self.notes.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a egress exception in status draft.
pub fn new_egress_exception(code: &str, title: &str) -> EgressException {
    EgressException {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        checksum: "".to_string(),
        burst: 0,
        rotation_days: 0,
        phone: "".to_string(),
        attempts: 0,
        tags: Vec::new(),
        notes: "".to_string(),
        unit: "".to_string(),
        status: EgressExceptionStatus::Draft,
    }
}
