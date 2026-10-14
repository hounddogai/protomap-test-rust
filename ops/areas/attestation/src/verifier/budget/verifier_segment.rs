use super::*;

use acme_kit::text;

/// Where a verifier segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierSegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl VerifierSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for VerifierSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier segment as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSegment {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub quantity: i64,
    pub score: f64,
    pub burst: i64,
    pub capacity: i64,
    pub label: String,
    pub status: VerifierSegmentStatus,
}

impl VerifierSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierSegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierSegmentStatus) -> bool {
        match self.status {
            VerifierSegmentStatus::Open => {
                match next {
                    VerifierSegmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierSegmentStatus::InReview => {
                return next == VerifierSegmentStatus::Reopened || next == VerifierSegmentStatus::Archived;
            }
            VerifierSegmentStatus::Resolved => {
                return next == VerifierSegmentStatus::Reopened || next == VerifierSegmentStatus::Archived;
            }
            VerifierSegmentStatus::Reopened => {
                return next == VerifierSegmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.burst as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierSegment {
        VerifierSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            quantity: self.quantity,
            score: self.score,
            burst: self.burst,
            capacity: self.capacity,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a verifier segment in status open.
pub fn new_verifier_segment(code: &str, name: &str) -> VerifierSegment {
    VerifierSegment {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        quantity: 0,
        score: 0.0,
        burst: 0,
        capacity: 0,
        label: "".to_string(),
        status: VerifierSegmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_verifier_segment("code-616", "name-371");
        assert_eq!(item.status, VerifierSegmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_verifier_segment("code-616", "name-371");
        if item.move_to(VerifierSegmentStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierSegmentStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_verifier_segment("code-616", "name-371");
        item.status = VerifierSegmentStatus::Archived;
        assert!(
            item.move_to(VerifierSegmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
