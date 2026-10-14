use super::*;

use acme_kit::text;

/// Where a zone exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneExceptionStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ZoneExceptionStatus {
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

impl std::fmt::Display for ZoneExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a zone exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneException {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub principal: String,
    pub issuer: String,
    pub quantity: i64,
    pub status: ZoneExceptionStatus,
}

impl ZoneException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneExceptionStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneExceptionStatus) -> bool {
        match self.status {
            ZoneExceptionStatus::Idle => {
                match next {
                    ZoneExceptionStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneExceptionStatus::Warming => {
                match next {
                    ZoneExceptionStatus::Ready | ZoneExceptionStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneExceptionStatus::Ready => {
                match next {
                    ZoneExceptionStatus::Draining | ZoneExceptionStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneExceptionStatus::Draining => {
                match next {
                    ZoneExceptionStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ZoneExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> ZoneException {
        ZoneException {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            principal: self.principal.clone(),
            issuer: self.issuer.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a zone exception in status idle.
pub fn new_zone_exception(code: &str, title: &str) -> ZoneException {
    ZoneException {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        principal: "".to_string(),
        issuer: "".to_string(),
        quantity: 0,
        status: ZoneExceptionStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_zone_exception("code-96", "title-750");
        assert_eq!(item.status, ZoneExceptionStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_zone_exception("code-96", "title-750");
        if item.move_to(ZoneExceptionStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneExceptionStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_zone_exception("code-96", "title-750");
        item.status = ZoneExceptionStatus::Stopped;
        assert!(
            item.move_to(ZoneExceptionStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
