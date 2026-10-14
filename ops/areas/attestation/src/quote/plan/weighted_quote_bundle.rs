use super::*;

use acme_kit::text;

/// Where a weighted quote bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedQuoteBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedQuoteBundleStatus {
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

impl std::fmt::Display for WeightedQuoteBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted quote bundle as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBundle {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub expires_at: i64,
    pub archived: bool,
    pub source: String,
    pub budget: f64,
    pub limit: i64,
    pub status: WeightedQuoteBundleStatus,
}

impl WeightedQuoteBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedQuoteBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedQuoteBundleStatus) -> bool {
        match self.status {
            WeightedQuoteBundleStatus::New => {
                return next == WeightedQuoteBundleStatus::Running || next == WeightedQuoteBundleStatus::Failed;
            }
            WeightedQuoteBundleStatus::Queued => {
                return next == WeightedQuoteBundleStatus::Done || next == WeightedQuoteBundleStatus::Failed;
            }
            WeightedQuoteBundleStatus::Running => {
                match next {
                    WeightedQuoteBundleStatus::Done | WeightedQuoteBundleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedQuoteBundleStatus::Done => {
                match next {
                    WeightedQuoteBundleStatus::Failed => {
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

    pub fn move_to(&mut self, next: WeightedQuoteBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted quote bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.budget + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedQuoteBundle {
        WeightedQuoteBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            expires_at: self.expires_at,
            archived: self.archived,
            source: self.source.clone(),
            budget: self.budget,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a weighted quote bundle in status new.
pub fn new_weighted_quote_bundle(code: &str, label: &str) -> WeightedQuoteBundle {
    WeightedQuoteBundle {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        expires_at: 0,
        archived: false,
        source: "".to_string(),
        budget: 0.0,
        limit: 0,
        status: WeightedQuoteBundleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_weighted_quote_bundle("code-708", "label-449");
        assert_eq!(item.status, WeightedQuoteBundleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_weighted_quote_bundle("code-708", "label-449");
        if item.move_to(WeightedQuoteBundleStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedQuoteBundleStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_weighted_quote_bundle("code-708", "label-449");
        item.status = WeightedQuoteBundleStatus::Failed;
        assert!(
            item.move_to(WeightedQuoteBundleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
