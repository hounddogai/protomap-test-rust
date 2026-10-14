use super::*;

use acme_kit::text;

/// Where a grant notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GrantNoticeStatus {
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

impl std::fmt::Display for GrantNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantNotice {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub namespace: String,
    pub ttl_seconds: i64,
    pub ceiling: f64,
    pub cipher: String,
    pub approved: bool,
    pub fingerprint: String,
    pub checksum: String,
    pub status: GrantNoticeStatus,
}

impl GrantNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantNoticeStatus) -> bool {
        match self.status {
            GrantNoticeStatus::New => {
                match next {
                    GrantNoticeStatus::Running | GrantNoticeStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantNoticeStatus::Queued => {
                match next {
                    GrantNoticeStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantNoticeStatus::Running => {
                return next == GrantNoticeStatus::Done || next == GrantNoticeStatus::Failed;
            }
            GrantNoticeStatus::Done => {
                match next {
                    GrantNoticeStatus::Failed => {
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

    pub fn move_to(&mut self, next: GrantNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "grant notice", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.ceiling * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantNotice {
        GrantNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            namespace: self.namespace.clone(),
            ttl_seconds: self.ttl_seconds,
            ceiling: self.ceiling,
            cipher: self.cipher.clone(),
            approved: self.approved,
            fingerprint: self.fingerprint.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a grant notice in status new.
pub fn new_grant_notice(code: &str, label: &str) -> GrantNotice {
    GrantNotice {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        namespace: "".to_string(),
        ttl_seconds: 0,
        ceiling: 0.0,
        cipher: "".to_string(),
        approved: false,
        fingerprint: "".to_string(),
        checksum: "".to_string(),
        status: GrantNoticeStatus::New,
    }
}
