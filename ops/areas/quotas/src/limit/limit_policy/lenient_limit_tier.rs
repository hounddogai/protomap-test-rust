use super::*;

use acme_kit::text;

/// Where a lenient limit tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientLimitTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientLimitTierStatus {
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

impl std::fmt::Display for LenientLimitTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient limit tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitTier {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub created_at: i64,
    pub rotation_days: i64,
    pub status: LenientLimitTierStatus,
}

impl LenientLimitTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientLimitTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientLimitTierStatus) -> bool {
        match self.status {
            LenientLimitTierStatus::Idle => {
                return next == LenientLimitTierStatus::Warming || next == LenientLimitTierStatus::Ready;
            }
            LenientLimitTierStatus::Warming => {
                match next {
                    LenientLimitTierStatus::Ready | LenientLimitTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientLimitTierStatus::Ready => {
                return next == LenientLimitTierStatus::Draining;
            }
            LenientLimitTierStatus::Draining => {
                return next == LenientLimitTierStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientLimitTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient limit tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.rotation_days as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientLimitTier {
        LenientLimitTier {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            created_at: self.created_at,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient limit tier in status idle.
pub fn new_lenient_limit_tier(code: &str, label: &str) -> LenientLimitTier {
    LenientLimitTier {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        created_at: 0,
        rotation_days: 0,
        status: LenientLimitTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_limit_tier("code-953", "label-223");
        assert_eq!(item.status, LenientLimitTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_lenient_limit_tier("code-953", "label-223");
        if item.move_to(LenientLimitTierStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientLimitTierStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_limit_tier("code-953", "label-223");
        item.status = LenientLimitTierStatus::Stopped;
        assert!(
            item.move_to(LenientLimitTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
