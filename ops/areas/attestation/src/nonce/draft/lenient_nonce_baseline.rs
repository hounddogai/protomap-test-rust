use super::*;

use acme_kit::text;

/// Where a lenient nonce baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientNonceBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientNonceBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LenientNonceBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient nonce baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceBaseline {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub rate: f64,
    pub weight: f64,
    pub status: LenientNonceBaselineStatus,
}

impl LenientNonceBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientNonceBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientNonceBaselineStatus) -> bool {
        match self.status {
            LenientNonceBaselineStatus::Planned => {
                return next == LenientNonceBaselineStatus::Completed;
            }
            LenientNonceBaselineStatus::Scheduled => {
                return next == LenientNonceBaselineStatus::InProgress;
            }
            LenientNonceBaselineStatus::InProgress => {
                return next == LenientNonceBaselineStatus::Completed || next == LenientNonceBaselineStatus::Cancelled;
            }
            LenientNonceBaselineStatus::Completed => {
                match next {
                    LenientNonceBaselineStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientNonceBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient nonce baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.rate - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientNonceBaseline {
        LenientNonceBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            rate: self.rate,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a lenient nonce baseline in status planned.
pub fn new_lenient_nonce_baseline(code: &str, title: &str) -> LenientNonceBaseline {
    LenientNonceBaseline {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        rate: 0.0,
        weight: 0.0,
        status: LenientNonceBaselineStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_nonce_baseline("code-484", "title-214");
        assert_eq!(item.status, LenientNonceBaselineStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_lenient_nonce_baseline("code-484", "title-214");
        if item.move_to(LenientNonceBaselineStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientNonceBaselineStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_nonce_baseline("code-484", "title-214");
        item.status = LenientNonceBaselineStatus::Cancelled;
        assert!(
            item.move_to(LenientNonceBaselineStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
