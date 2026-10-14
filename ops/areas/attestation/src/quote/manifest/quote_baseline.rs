use super::*;

use acme_kit::text;

/// Where a quote baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl QuoteBaselineStatus {
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

impl std::fmt::Display for QuoteBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBaseline {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub margin: f64,
    pub score: f64,
    pub window_days: i64,
    pub rotation_days: i64,
    pub maximum: f64,
    pub channel: String,
    pub status: QuoteBaselineStatus,
}

impl QuoteBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteBaselineStatus) -> bool {
        match self.status {
            QuoteBaselineStatus::Planned => {
                return next == QuoteBaselineStatus::InProgress || next == QuoteBaselineStatus::Completed;
            }
            QuoteBaselineStatus::Scheduled => {
                match next {
                    QuoteBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteBaselineStatus::InProgress => {
                match next {
                    QuoteBaselineStatus::Completed | QuoteBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteBaselineStatus::Completed => {
                match next {
                    QuoteBaselineStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: QuoteBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.score - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteBaseline {
        QuoteBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            margin: self.margin,
            score: self.score,
            window_days: self.window_days,
            rotation_days: self.rotation_days,
            maximum: self.maximum,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a quote baseline in status planned.
pub fn new_quote_baseline(code: &str, title: &str) -> QuoteBaseline {
    QuoteBaseline {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        margin: 0.0,
        score: 0.0,
        window_days: 0,
        rotation_days: 0,
        maximum: 0.0,
        channel: "".to_string(),
        status: QuoteBaselineStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_quote_baseline("code-559", "title-843");
        assert_eq!(item.status, QuoteBaselineStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_quote_baseline("code-559", "title-843");
        if item.move_to(QuoteBaselineStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteBaselineStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_quote_baseline("code-559", "title-843");
        item.status = QuoteBaselineStatus::Cancelled;
        assert!(
            item.move_to(QuoteBaselineStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
