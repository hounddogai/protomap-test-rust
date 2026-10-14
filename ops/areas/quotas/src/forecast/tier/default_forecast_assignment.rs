use super::*;

use acme_kit::text;

/// Where a default forecast assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultForecastAssignmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultForecastAssignmentStatus {
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

impl std::fmt::Display for DefaultForecastAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default forecast assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastAssignment {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub reviewer: String,
    pub active: bool,
    pub amount: f64,
    pub status: DefaultForecastAssignmentStatus,
}

impl DefaultForecastAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultForecastAssignmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultForecastAssignmentStatus) -> bool {
        match self.status {
            DefaultForecastAssignmentStatus::Idle => {
                return next == DefaultForecastAssignmentStatus::Draining
                    || next == DefaultForecastAssignmentStatus::Stopped;
            }
            DefaultForecastAssignmentStatus::Warming => {
                return next == DefaultForecastAssignmentStatus::Draining
                    || next == DefaultForecastAssignmentStatus::Stopped;
            }
            DefaultForecastAssignmentStatus::Ready => {
                return next == DefaultForecastAssignmentStatus::Stopped;
            }
            DefaultForecastAssignmentStatus::Draining => {
                match next {
                    DefaultForecastAssignmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultForecastAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default forecast assignment {} cannot move from {} to {}",
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
        self.amount
    }

    pub fn copy(&self) -> DefaultForecastAssignment {
        DefaultForecastAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            reviewer: self.reviewer.clone(),
            active: self.active,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default forecast assignment in status idle.
pub fn new_default_forecast_assignment(code: &str, title: &str) -> DefaultForecastAssignment {
    DefaultForecastAssignment {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        reviewer: "".to_string(),
        active: false,
        amount: 0.0,
        status: DefaultForecastAssignmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_forecast_assignment("code-238", "title-401");
        assert_eq!(item.status, DefaultForecastAssignmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_default_forecast_assignment("code-238", "title-401");
        if item.move_to(DefaultForecastAssignmentStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultForecastAssignmentStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_forecast_assignment("code-238", "title-401");
        item.status = DefaultForecastAssignmentStatus::Stopped;
        assert!(
            item.move_to(DefaultForecastAssignmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
