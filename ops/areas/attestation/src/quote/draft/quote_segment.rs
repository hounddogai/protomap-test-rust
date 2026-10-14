use super::*;

use acme_kit::text;

/// Where a quote segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QuoteSegmentStatus {
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

impl std::fmt::Display for QuoteSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One quote segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSegment {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub window_days: i64,
    pub rate: f64,
    pub status: QuoteSegmentStatus,
}

impl QuoteSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteSegmentStatus) -> bool {
        match self.status {
            QuoteSegmentStatus::Idle => {
                return next == QuoteSegmentStatus::Warming || next == QuoteSegmentStatus::Draining;
            }
            QuoteSegmentStatus::Warming => {
                match next {
                    QuoteSegmentStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteSegmentStatus::Ready => {
                match next {
                    QuoteSegmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteSegmentStatus::Draining => {
                match next {
                    QuoteSegmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: QuoteSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.window_days as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteSegment {
        QuoteSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            window_days: self.window_days,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a quote segment in status idle.
pub fn new_quote_segment(code: &str, title: &str) -> QuoteSegment {
    QuoteSegment {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        window_days: 0,
        rate: 0.0,
        status: QuoteSegmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_quote_segment("code-530", "title-414");
        assert_eq!(item.status, QuoteSegmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_quote_segment("code-530", "title-414");
        if item.move_to(QuoteSegmentStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteSegmentStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_quote_segment("code-530", "title-414");
        item.status = QuoteSegmentStatus::Stopped;
        assert!(item.move_to(QuoteSegmentStatus::Idle).is_err(), "expected move to fail");
    }
}
