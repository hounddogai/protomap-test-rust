use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a overage exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageExceptionStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl OverageExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for OverageExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One overage exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageException {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub maximum: f64,
    pub reviewer: String,
    pub key_id: String,
    pub rotation_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub category: String,
    pub country: String,
    pub created_at: i64,
    pub status: OverageExceptionStatus,
}

impl OverageException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageExceptionStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageExceptionStatus) -> bool {
        match self.status {
            OverageExceptionStatus::New => {
                match next {
                    OverageExceptionStatus::Running | OverageExceptionStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageExceptionStatus::Queued => {
                return next == OverageExceptionStatus::Running || next == OverageExceptionStatus::Failed;
            }
            OverageExceptionStatus::Running => {
                return next == OverageExceptionStatus::Done || next == OverageExceptionStatus::Failed;
            }
            OverageExceptionStatus::Done => {
                match next {
                    OverageExceptionStatus::Failed => {
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

    pub fn move_to(&mut self, next: OverageExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage exception {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn adjusted_load(&self) -> f64 {
        ((self.created_at as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageException {
        OverageException {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            maximum: self.maximum,
            reviewer: self.reviewer.clone(),
            key_id: self.key_id.clone(),
            rotation_days: self.rotation_days,
            attributes: self.attributes.clone(),
            category: self.category.clone(),
            country: self.country.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a overage exception in status new.
pub fn new_overage_exception(code: &str, name: &str) -> OverageException {
    OverageException {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        maximum: 0.0,
        reviewer: "".to_string(),
        key_id: "".to_string(),
        rotation_days: 0,
        attributes: BTreeMap::new(),
        category: "".to_string(),
        country: "".to_string(),
        created_at: 0,
        status: OverageExceptionStatus::New,
    }
}
