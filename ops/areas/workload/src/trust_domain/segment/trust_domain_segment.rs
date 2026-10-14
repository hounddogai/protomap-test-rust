use super::*;

use acme_kit::text;

/// Where a trust domain segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TrustDomainSegmentStatus {
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

impl std::fmt::Display for TrustDomainSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a trust domain segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSegment {
    pub code: String,
    pub title: String,
    pub host: String,
    pub owner_id: String,
    pub budget: f64,
    pub key_id: String,
    pub status: TrustDomainSegmentStatus,
}

impl TrustDomainSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainSegmentStatus) -> bool {
        match self.status {
            TrustDomainSegmentStatus::Planned => {
                match next {
                    TrustDomainSegmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainSegmentStatus::Scheduled => {
                return next == TrustDomainSegmentStatus::Completed;
            }
            TrustDomainSegmentStatus::InProgress => {
                match next {
                    TrustDomainSegmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainSegmentStatus::Completed => {
                return next == TrustDomainSegmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "trust domain segment", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> TrustDomainSegment {
        TrustDomainSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            owner_id: self.owner_id.clone(),
            budget: self.budget,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a trust domain segment in status planned.
pub fn new_trust_domain_segment(code: &str, title: &str) -> TrustDomainSegment {
    TrustDomainSegment {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        owner_id: "".to_string(),
        budget: 0.0,
        key_id: "".to_string(),
        status: TrustDomainSegmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_trust_domain_segment("code-450", "title-235");
        assert_eq!(item.status, TrustDomainSegmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_trust_domain_segment("code-450", "title-235");
        if item.move_to(TrustDomainSegmentStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainSegmentStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_trust_domain_segment("code-450", "title-235");
        item.status = TrustDomainSegmentStatus::Cancelled;
        assert!(
            item.move_to(TrustDomainSegmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
