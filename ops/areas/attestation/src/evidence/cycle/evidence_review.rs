use super::*;

use acme_kit::text;

/// Where a evidence review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EvidenceReviewStatus {
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

impl std::fmt::Display for EvidenceReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceReview {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub name: String,
    pub grace_days: i64,
    pub host: String,
    pub status: EvidenceReviewStatus,
}

impl EvidenceReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceReviewStatus) -> bool {
        match self.status {
            EvidenceReviewStatus::Idle => {
                match next {
                    EvidenceReviewStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceReviewStatus::Warming => {
                match next {
                    EvidenceReviewStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceReviewStatus::Ready => {
                match next {
                    EvidenceReviewStatus::Draining | EvidenceReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceReviewStatus::Draining => {
                match next {
                    EvidenceReviewStatus::Stopped => {
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

    pub fn move_to(&mut self, next: EvidenceReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> EvidenceReview {
        EvidenceReview {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a evidence review in status idle.
pub fn new_evidence_review(code: &str, title: &str) -> EvidenceReview {
    EvidenceReview {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        name: "".to_string(),
        grace_days: 0,
        host: "".to_string(),
        status: EvidenceReviewStatus::Idle,
    }
}
