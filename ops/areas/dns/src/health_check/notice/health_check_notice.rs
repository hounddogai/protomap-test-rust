use super::*;

use acme_kit::text;

/// Where a health check notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl HealthCheckNoticeStatus {
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

impl std::fmt::Display for HealthCheckNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check notice as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckNotice {
    pub code: String,
    pub label: String,
    pub category: String,
    pub attempts: i64,
    pub version: i64,
    pub tags: Vec<String>,
    pub status: HealthCheckNoticeStatus,
}

impl HealthCheckNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckNoticeStatus) -> bool {
        match self.status {
            HealthCheckNoticeStatus::New => {
                return next == HealthCheckNoticeStatus::Running;
            }
            HealthCheckNoticeStatus::Queued => {
                match next {
                    HealthCheckNoticeStatus::Running | HealthCheckNoticeStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckNoticeStatus::Running => {
                match next {
                    HealthCheckNoticeStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckNoticeStatus::Done => {
                match next {
                    HealthCheckNoticeStatus::Failed => {
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

    pub fn move_to(&mut self, next: HealthCheckNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
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

    pub fn weighted_total(&self) -> f64 {
        ((self.version as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckNotice {
        HealthCheckNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            attempts: self.attempts,
            version: self.version,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a health check notice in status new.
pub fn new_health_check_notice(code: &str, label: &str) -> HealthCheckNotice {
    HealthCheckNotice {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        attempts: 0,
        version: 0,
        tags: Vec::new(),
        status: HealthCheckNoticeStatus::New,
    }
}
