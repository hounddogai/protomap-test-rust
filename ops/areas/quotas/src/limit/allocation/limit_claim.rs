use super::*;

use acme_kit::text;

/// Where a limit claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitClaimStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LimitClaimStatus {
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

impl std::fmt::Display for LimitClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit claim as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitClaim {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub margin: f64,
    pub category: String,
    pub region: String,
    pub amount: f64,
    pub unit: String,
    pub status: LimitClaimStatus,
}

impl LimitClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitClaimStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitClaimStatus) -> bool {
        match self.status {
            LimitClaimStatus::Idle => {
                return next == LimitClaimStatus::Stopped;
            }
            LimitClaimStatus::Warming => {
                match next {
                    LimitClaimStatus::Ready | LimitClaimStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitClaimStatus::Ready => {
                match next {
                    LimitClaimStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitClaimStatus::Draining => {
                return next == LimitClaimStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        (self.amount * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitClaim {
        LimitClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            margin: self.margin,
            category: self.category.clone(),
            region: self.region.clone(),
            amount: self.amount,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a limit claim in status idle.
pub fn new_limit_claim(code: &str, title: &str) -> LimitClaim {
    LimitClaim {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        margin: 0.0,
        category: "".to_string(),
        region: "".to_string(),
        amount: 0.0,
        unit: "".to_string(),
        status: LimitClaimStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_limit_claim("code-465", "title-785");
        assert_eq!(item.status, LimitClaimStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_limit_claim("code-465", "title-785");
        if item.move_to(LimitClaimStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitClaimStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_limit_claim("code-465", "title-785");
        item.status = LimitClaimStatus::Stopped;
        assert!(item.move_to(LimitClaimStatus::Idle).is_err(), "expected move to fail");
    }
}
