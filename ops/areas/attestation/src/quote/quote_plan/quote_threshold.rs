use super::*;

use acme_kit::text;

/// Where a quote threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteThresholdStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QuoteThresholdStatus {
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

impl std::fmt::Display for QuoteThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One quote threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteThreshold {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub starts_at: i64,
    pub locale: String,
    pub segment: String,
    pub attempts: i64,
    pub status: QuoteThresholdStatus,
}

impl QuoteThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteThresholdStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteThresholdStatus) -> bool {
        match self.status {
            QuoteThresholdStatus::Idle => {
                match next {
                    QuoteThresholdStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteThresholdStatus::Warming => {
                return next == QuoteThresholdStatus::Ready || next == QuoteThresholdStatus::Stopped;
            }
            QuoteThresholdStatus::Ready => {
                return next == QuoteThresholdStatus::Stopped;
            }
            QuoteThresholdStatus::Draining => {
                match next {
                    QuoteThresholdStatus::Stopped => {
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

    pub fn move_to(&mut self, next: QuoteThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "quote threshold", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.starts_at as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteThreshold {
        QuoteThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            starts_at: self.starts_at,
            locale: self.locale.clone(),
            segment: self.segment.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a quote threshold in status idle.
pub fn new_quote_threshold(code: &str, title: &str) -> QuoteThreshold {
    QuoteThreshold {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        starts_at: 0,
        locale: "".to_string(),
        segment: "".to_string(),
        attempts: 0,
        status: QuoteThresholdStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_quote_threshold("code-860", "title-302");
        assert_eq!(item.status, QuoteThresholdStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_quote_threshold("code-860", "title-302");
        if item.move_to(QuoteThresholdStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteThresholdStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_quote_threshold("code-860", "title-302");
        item.status = QuoteThresholdStatus::Stopped;
        assert!(
            item.move_to(QuoteThresholdStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
