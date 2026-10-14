use super::*;

use acme_kit::text;

/// Where a zone claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneClaimStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ZoneClaimStatus {
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

impl std::fmt::Display for ZoneClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a zone claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneClaim {
    pub code: String,
    pub title: String,
    pub country: String,
    pub capacity: i64,
    pub budget: f64,
    pub owner_id: String,
    pub status: ZoneClaimStatus,
}

impl ZoneClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneClaimStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneClaimStatus) -> bool {
        match self.status {
            ZoneClaimStatus::Idle => {
                match next {
                    ZoneClaimStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneClaimStatus::Warming => {
                match next {
                    ZoneClaimStatus::Draining | ZoneClaimStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneClaimStatus::Ready => {
                match next {
                    ZoneClaimStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneClaimStatus::Draining => {
                match next {
                    ZoneClaimStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ZoneClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone claim {} cannot move from {} to {}",
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
        ((self.capacity as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneClaim {
        ZoneClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            capacity: self.capacity,
            budget: self.budget,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a zone claim in status idle.
pub fn new_zone_claim(code: &str, title: &str) -> ZoneClaim {
    ZoneClaim {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        capacity: 0,
        budget: 0.0,
        owner_id: "".to_string(),
        status: ZoneClaimStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_zone_claim("code-101", "title-317");
        assert_eq!(item.status, ZoneClaimStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_zone_claim("code-101", "title-317");
        if item.move_to(ZoneClaimStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneClaimStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_zone_claim("code-101", "title-317");
        item.status = ZoneClaimStatus::Stopped;
        assert!(item.move_to(ZoneClaimStatus::Idle).is_err(), "expected move to fail");
    }
}
