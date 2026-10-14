use super::*;

use acme_kit::text;

/// Where a quote estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl QuoteEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for QuoteEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a quote estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteEstimate {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub ends_at: i64,
    pub starts_at: i64,
    pub status: QuoteEstimateStatus,
}

impl QuoteEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteEstimateStatus) -> bool {
        match self.status {
            QuoteEstimateStatus::Created => {
                return next == QuoteEstimateStatus::Published;
            }
            QuoteEstimateStatus::Validated => {
                match next {
                    QuoteEstimateStatus::Published | QuoteEstimateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteEstimateStatus::Published => {
                return next == QuoteEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote estimate {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "quote estimate", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.batch_size as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteEstimate {
        QuoteEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            ends_at: self.ends_at,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a quote estimate in status created.
pub fn new_quote_estimate(code: &str, title: &str) -> QuoteEstimate {
    QuoteEstimate {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        ends_at: 0,
        starts_at: 0,
        status: QuoteEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_quote_estimate("code-254", "title-905");
        assert_eq!(item.status, QuoteEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_quote_estimate("code-254", "title-905");
        if item.move_to(QuoteEstimateStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteEstimateStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_quote_estimate("code-254", "title-905");
        item.status = QuoteEstimateStatus::Retired;
        assert!(
            item.move_to(QuoteEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
