use super::*;

use acme_kit::text;

/// Where a fallback segment proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackSegmentProposalStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackSegmentProposalStatus {
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

impl std::fmt::Display for FallbackSegmentProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback segment proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentProposal {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub summary: String,
    pub fingerprint: String,
    pub attempts: i64,
    pub duration_minutes: i64,
    pub rotation_days: i64,
    pub status: FallbackSegmentProposalStatus,
}

impl FallbackSegmentProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackSegmentProposalStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackSegmentProposalStatus) -> bool {
        match self.status {
            FallbackSegmentProposalStatus::Idle => {
                match next {
                    FallbackSegmentProposalStatus::Ready | FallbackSegmentProposalStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSegmentProposalStatus::Warming => {
                match next {
                    FallbackSegmentProposalStatus::Ready | FallbackSegmentProposalStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSegmentProposalStatus::Ready => {
                match next {
                    FallbackSegmentProposalStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSegmentProposalStatus::Draining => {
                return next == FallbackSegmentProposalStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackSegmentProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback segment proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ceiling + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackSegmentProposal {
        FallbackSegmentProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            summary: self.summary.clone(),
            fingerprint: self.fingerprint.clone(),
            attempts: self.attempts,
            duration_minutes: self.duration_minutes,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback segment proposal in status idle.
pub fn new_fallback_segment_proposal(code: &str, title: &str) -> FallbackSegmentProposal {
    FallbackSegmentProposal {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        summary: "".to_string(),
        fingerprint: "".to_string(),
        attempts: 0,
        duration_minutes: 0,
        rotation_days: 0,
        status: FallbackSegmentProposalStatus::Idle,
    }
}
