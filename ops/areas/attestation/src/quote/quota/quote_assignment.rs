use super::*;

use acme_kit::text;

/// Where a quote assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteAssignmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QuoteAssignmentStatus {
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

impl std::fmt::Display for QuoteAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One quote assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAssignment {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub external_ref: String,
    pub ceiling: f64,
    pub region: String,
    pub status: QuoteAssignmentStatus,
}

impl QuoteAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteAssignmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteAssignmentStatus) -> bool {
        match self.status {
            QuoteAssignmentStatus::Idle => {
                return next == QuoteAssignmentStatus::Ready || next == QuoteAssignmentStatus::Stopped;
            }
            QuoteAssignmentStatus::Warming => {
                return next == QuoteAssignmentStatus::Ready;
            }
            QuoteAssignmentStatus::Ready => {
                match next {
                    QuoteAssignmentStatus::Draining | QuoteAssignmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteAssignmentStatus::Draining => {
                match next {
                    QuoteAssignmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: QuoteAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> QuoteAssignment {
        QuoteAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            external_ref: self.external_ref.clone(),
            ceiling: self.ceiling,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a quote assignment in status idle.
pub fn new_quote_assignment(code: &str, title: &str) -> QuoteAssignment {
    QuoteAssignment {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        external_ref: "".to_string(),
        ceiling: 0.0,
        region: "".to_string(),
        status: QuoteAssignmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_quote_assignment("code-811", "title-462");
        assert_eq!(item.status, QuoteAssignmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_quote_assignment("code-811", "title-462");
        if item.move_to(QuoteAssignmentStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteAssignmentStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_quote_assignment("code-811", "title-462");
        item.status = QuoteAssignmentStatus::Stopped;
        assert!(
            item.move_to(QuoteAssignmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
