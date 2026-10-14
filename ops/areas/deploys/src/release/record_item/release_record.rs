use super::*;

use acme_kit::text;

/// Where a release record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseRecordStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReleaseRecordStatus {
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

impl std::fmt::Display for ReleaseRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One release record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRecord {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub principal: String,
    pub phone: String,
    pub approved: bool,
    pub status: ReleaseRecordStatus,
}

impl ReleaseRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseRecordStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseRecordStatus) -> bool {
        match self.status {
            ReleaseRecordStatus::Idle => {
                return next == ReleaseRecordStatus::Ready;
            }
            ReleaseRecordStatus::Warming => {
                match next {
                    ReleaseRecordStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseRecordStatus::Ready => {
                return next == ReleaseRecordStatus::Draining;
            }
            ReleaseRecordStatus::Draining => {
                match next {
                    ReleaseRecordStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ReleaseRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> ReleaseRecord {
        ReleaseRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            principal: self.principal.clone(),
            phone: self.phone.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a release record in status idle.
pub fn new_release_record(code: &str, name: &str) -> ReleaseRecord {
    ReleaseRecord {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        principal: "".to_string(),
        phone: "".to_string(),
        approved: false,
        status: ReleaseRecordStatus::Idle,
    }
}
