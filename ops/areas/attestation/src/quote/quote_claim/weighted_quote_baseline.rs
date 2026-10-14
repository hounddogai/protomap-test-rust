use super::*;

use acme_kit::text;

/// Where a weighted quote baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedQuoteBaselineStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedQuoteBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for WeightedQuoteBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted quote baseline as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBaseline {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub summary: String,
    pub region: String,
    pub status: WeightedQuoteBaselineStatus,
}

impl WeightedQuoteBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedQuoteBaselineStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedQuoteBaselineStatus) -> bool {
        match self.status {
            WeightedQuoteBaselineStatus::New => {
                match next {
                    WeightedQuoteBaselineStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedQuoteBaselineStatus::Queued => {
                return next == WeightedQuoteBaselineStatus::Running;
            }
            WeightedQuoteBaselineStatus::Running => {
                match next {
                    WeightedQuoteBaselineStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedQuoteBaselineStatus::Done => {
                return next == WeightedQuoteBaselineStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedQuoteBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted quote baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> WeightedQuoteBaseline {
        WeightedQuoteBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            summary: self.summary.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a weighted quote baseline in status new.
pub fn new_weighted_quote_baseline(code: &str, name: &str) -> WeightedQuoteBaseline {
    WeightedQuoteBaseline {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        summary: "".to_string(),
        region: "".to_string(),
        status: WeightedQuoteBaselineStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_weighted_quote_baseline("code-557", "name-323");
        assert_eq!(item.status, WeightedQuoteBaselineStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_weighted_quote_baseline("code-557", "name-323");
        if item.move_to(WeightedQuoteBaselineStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedQuoteBaselineStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_weighted_quote_baseline("code-557", "name-323");
        item.status = WeightedQuoteBaselineStatus::Failed;
        assert!(
            item.move_to(WeightedQuoteBaselineStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
