use super::*;

use acme_kit::text;

/// Where a isolation claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationClaimStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl IsolationClaimStatus {
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

impl std::fmt::Display for IsolationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a isolation claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationClaim {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub port: i64,
    pub cipher: String,
    pub approved: bool,
    pub scope: String,
    pub status: IsolationClaimStatus,
}

impl IsolationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationClaimStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationClaimStatus) -> bool {
        match self.status {
            IsolationClaimStatus::Idle => {
                return next == IsolationClaimStatus::Ready || next == IsolationClaimStatus::Draining;
            }
            IsolationClaimStatus::Warming => {
                match next {
                    IsolationClaimStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationClaimStatus::Ready => {
                match next {
                    IsolationClaimStatus::Draining | IsolationClaimStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationClaimStatus::Draining => {
                match next {
                    IsolationClaimStatus::Stopped => {
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

    pub fn move_to(&mut self, next: IsolationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.rotation_days as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationClaim {
        IsolationClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            port: self.port,
            cipher: self.cipher.clone(),
            approved: self.approved,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a isolation claim in status idle.
pub fn new_isolation_claim(code: &str, title: &str) -> IsolationClaim {
    IsolationClaim {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        port: 0,
        cipher: "".to_string(),
        approved: false,
        scope: "".to_string(),
        status: IsolationClaimStatus::Idle,
    }
}
