use super::*;

use acme_kit::text;

/// Where a freeze exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeExceptionStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FreezeExceptionStatus {
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

impl std::fmt::Display for FreezeExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a freeze exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeException {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub timezone: String,
    pub unit: String,
    pub checksum: String,
    pub budget: f64,
    pub burst: i64,
    pub rotation_days: i64,
    pub status: FreezeExceptionStatus,
}

impl FreezeException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeExceptionStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeExceptionStatus) -> bool {
        match self.status {
            FreezeExceptionStatus::Idle => {
                match next {
                    FreezeExceptionStatus::Warming | FreezeExceptionStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeExceptionStatus::Warming => {
                match next {
                    FreezeExceptionStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeExceptionStatus::Ready => {
                match next {
                    FreezeExceptionStatus::Draining | FreezeExceptionStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeExceptionStatus::Draining => {
                return next == FreezeExceptionStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.rate * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeException {
        FreezeException {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            timezone: self.timezone.clone(),
            unit: self.unit.clone(),
            checksum: self.checksum.clone(),
            budget: self.budget,
            burst: self.burst,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a freeze exception in status idle.
pub fn new_freeze_exception(code: &str, title: &str) -> FreezeException {
    FreezeException {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        timezone: "".to_string(),
        unit: "".to_string(),
        checksum: "".to_string(),
        budget: 0.0,
        burst: 0,
        rotation_days: 0,
        status: FreezeExceptionStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_freeze_exception("code-750", "title-548");
        assert_eq!(item.status, FreezeExceptionStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_freeze_exception("code-750", "title-548");
        if item.move_to(FreezeExceptionStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeExceptionStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_freeze_exception("code-750", "title-548");
        item.status = FreezeExceptionStatus::Stopped;
        assert!(
            item.move_to(FreezeExceptionStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
