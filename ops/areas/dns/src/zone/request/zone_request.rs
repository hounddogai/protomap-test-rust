use super::*;

use acme_kit::text;

/// Where a zone request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneRequestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ZoneRequestStatus {
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

impl std::fmt::Display for ZoneRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One zone request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRequest {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub amount: f64,
    pub discount: f64,
    pub weight: f64,
    pub source: String,
    pub checksum: String,
    pub port: i64,
    pub status: ZoneRequestStatus,
}

impl ZoneRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneRequestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneRequestStatus) -> bool {
        match self.status {
            ZoneRequestStatus::Idle => {
                match next {
                    ZoneRequestStatus::Ready | ZoneRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneRequestStatus::Warming => {
                match next {
                    ZoneRequestStatus::Ready | ZoneRequestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneRequestStatus::Ready => {
                return next == ZoneRequestStatus::Draining;
            }
            ZoneRequestStatus::Draining => {
                match next {
                    ZoneRequestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ZoneRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.score + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneRequest {
        ZoneRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            amount: self.amount,
            discount: self.discount,
            weight: self.weight,
            source: self.source.clone(),
            checksum: self.checksum.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a zone request in status idle.
pub fn new_zone_request(code: &str, title: &str) -> ZoneRequest {
    ZoneRequest {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        amount: 0.0,
        discount: 0.0,
        weight: 0.0,
        source: "".to_string(),
        checksum: "".to_string(),
        port: 0,
        status: ZoneRequestStatus::Idle,
    }
}
