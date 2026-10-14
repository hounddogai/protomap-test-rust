use super::*;

use acme_kit::text;

/// Where a limit quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LimitQuotaStatus {
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

impl std::fmt::Display for LimitQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitQuota {
    pub code: String,
    pub title: String,
    pub region: String,
    pub approved: bool,
    pub active: bool,
    pub amount: f64,
    pub status: LimitQuotaStatus,
}

impl LimitQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitQuotaStatus) -> bool {
        match self.status {
            LimitQuotaStatus::Idle => {
                match next {
                    LimitQuotaStatus::Warming | LimitQuotaStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitQuotaStatus::Warming => {
                match next {
                    LimitQuotaStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitQuotaStatus::Ready => {
                return next == LimitQuotaStatus::Draining || next == LimitQuotaStatus::Stopped;
            }
            LimitQuotaStatus::Draining => {
                return next == LimitQuotaStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit quota {} cannot move from {} to {}",
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
        self.amount
    }

    pub fn copy(&self) -> LimitQuota {
        LimitQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            approved: self.approved,
            active: self.active,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a limit quota in status idle.
pub fn new_limit_quota(code: &str, title: &str) -> LimitQuota {
    LimitQuota {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        approved: false,
        active: false,
        amount: 0.0,
        status: LimitQuotaStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_limit_quota("code-906", "title-543");
        assert_eq!(item.status, LimitQuotaStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_limit_quota("code-906", "title-543");
        if item.move_to(LimitQuotaStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitQuotaStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_limit_quota("code-906", "title-543");
        item.status = LimitQuotaStatus::Stopped;
        assert!(item.move_to(LimitQuotaStatus::Idle).is_err(), "expected move to fail");
    }
}
