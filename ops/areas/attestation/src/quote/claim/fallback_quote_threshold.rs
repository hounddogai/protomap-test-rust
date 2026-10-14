use super::*;

use acme_kit::text;

/// Where a fallback quote threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackQuoteThresholdStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackQuoteThresholdStatus {
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

impl std::fmt::Display for FallbackQuoteThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback quote threshold as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteThreshold {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub issuer: String,
    pub floor_value: f64,
    pub margin: f64,
    pub version: i64,
    pub status: FallbackQuoteThresholdStatus,
}

impl FallbackQuoteThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackQuoteThresholdStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackQuoteThresholdStatus) -> bool {
        match self.status {
            FallbackQuoteThresholdStatus::Planned => {
                return next == FallbackQuoteThresholdStatus::Scheduled
                    || next == FallbackQuoteThresholdStatus::Completed;
            }
            FallbackQuoteThresholdStatus::Scheduled => {
                return next == FallbackQuoteThresholdStatus::Completed
                    || next == FallbackQuoteThresholdStatus::Cancelled;
            }
            FallbackQuoteThresholdStatus::InProgress => {
                match next {
                    FallbackQuoteThresholdStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackQuoteThresholdStatus::Completed => {
                return next == FallbackQuoteThresholdStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackQuoteThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback quote threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.margin * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackQuoteThreshold {
        FallbackQuoteThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            issuer: self.issuer.clone(),
            floor_value: self.floor_value,
            margin: self.margin,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback quote threshold in status planned.
pub fn new_fallback_quote_threshold(code: &str, name: &str) -> FallbackQuoteThreshold {
    FallbackQuoteThreshold {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        issuer: "".to_string(),
        floor_value: 0.0,
        margin: 0.0,
        version: 0,
        status: FallbackQuoteThresholdStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_quote_threshold("code-915", "name-286");
        assert_eq!(item.status, FallbackQuoteThresholdStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_fallback_quote_threshold("code-915", "name-286");
        if item.move_to(FallbackQuoteThresholdStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackQuoteThresholdStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_quote_threshold("code-915", "name-286");
        item.status = FallbackQuoteThresholdStatus::Cancelled;
        assert!(
            item.move_to(FallbackQuoteThresholdStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
