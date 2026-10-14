use super::*;

use acme_kit::text;

/// Where a fallback page variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPageVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackPageVarianceStatus {
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

impl std::fmt::Display for FallbackPageVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback page variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageVariance {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub summary: String,
    pub rotation_days: i64,
    pub rate: f64,
    pub sequence: i64,
    pub status: FallbackPageVarianceStatus,
}

impl FallbackPageVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPageVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPageVarianceStatus) -> bool {
        match self.status {
            FallbackPageVarianceStatus::Idle => {
                match next {
                    FallbackPageVarianceStatus::Ready | FallbackPageVarianceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPageVarianceStatus::Warming => {
                return next == FallbackPageVarianceStatus::Draining || next == FallbackPageVarianceStatus::Stopped;
            }
            FallbackPageVarianceStatus::Ready => {
                return next == FallbackPageVarianceStatus::Draining || next == FallbackPageVarianceStatus::Stopped;
            }
            FallbackPageVarianceStatus::Draining => {
                return next == FallbackPageVarianceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPageVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback page variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.rotation_days as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPageVariance {
        FallbackPageVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            summary: self.summary.clone(),
            rotation_days: self.rotation_days,
            rate: self.rate,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback page variance in status idle.
pub fn new_fallback_page_variance(code: &str, name: &str) -> FallbackPageVariance {
    FallbackPageVariance {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        summary: "".to_string(),
        rotation_days: 0,
        rate: 0.0,
        sequence: 0,
        status: FallbackPageVarianceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_page_variance("code-386", "name-115");
        assert_eq!(item.status, FallbackPageVarianceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_fallback_page_variance("code-386", "name-115");
        if item.move_to(FallbackPageVarianceStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPageVarianceStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_page_variance("code-386", "name-115");
        item.status = FallbackPageVarianceStatus::Stopped;
        assert!(
            item.move_to(FallbackPageVarianceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
