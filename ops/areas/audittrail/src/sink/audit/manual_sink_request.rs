use super::*;

use acme_kit::text;

/// Where a manual sink request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualSinkRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualSinkRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualSinkRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual sink request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkRequest {
    pub code: String,
    pub title: String,
    pub description: String,
    pub floor_value: f64,
    pub segment: String,
    pub status: ManualSinkRequestStatus,
}

impl ManualSinkRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualSinkRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualSinkRequestStatus) -> bool {
        match self.status {
            ManualSinkRequestStatus::Requested => {
                match next {
                    ManualSinkRequestStatus::Granted | ManualSinkRequestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSinkRequestStatus::Granted => {
                match next {
                    ManualSinkRequestStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSinkRequestStatus::Revoked => {
                return next == ManualSinkRequestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualSinkRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual sink request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.floor_value
    }

    pub fn copy(&self) -> ManualSinkRequest {
        ManualSinkRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            floor_value: self.floor_value,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a manual sink request in status requested.
pub fn new_manual_sink_request(code: &str, title: &str) -> ManualSinkRequest {
    ManualSinkRequest {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        floor_value: 0.0,
        segment: "".to_string(),
        status: ManualSinkRequestStatus::Requested,
    }
}
