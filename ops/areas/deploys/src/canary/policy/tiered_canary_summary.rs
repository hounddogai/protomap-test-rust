use super::*;

use acme_kit::text;

/// Where a tiered canary summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredCanarySummaryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredCanarySummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredCanarySummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered canary summary as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanarySummary {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub source: String,
    pub category: String,
    pub cost: f64,
    pub ratio: f64,
    pub status: TieredCanarySummaryStatus,
}

impl TieredCanarySummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredCanarySummaryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredCanarySummaryStatus) -> bool {
        match self.status {
            TieredCanarySummaryStatus::Created => {
                match next {
                    TieredCanarySummaryStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredCanarySummaryStatus::Validated => {
                match next {
                    TieredCanarySummaryStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredCanarySummaryStatus::Published => {
                match next {
                    TieredCanarySummaryStatus::Retired => {
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

    pub fn move_to(&mut self, next: TieredCanarySummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered canary summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered canary summary", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        (self.cost * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredCanarySummary {
        TieredCanarySummary {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            source: self.source.clone(),
            category: self.category.clone(),
            cost: self.cost,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tiered canary summary in status created.
pub fn new_tiered_canary_summary(code: &str, label: &str) -> TieredCanarySummary {
    TieredCanarySummary {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        source: "".to_string(),
        category: "".to_string(),
        cost: 0.0,
        ratio: 0.0,
        status: TieredCanarySummaryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_canary_summary("code-745", "label-627");
        assert_eq!(item.status, TieredCanarySummaryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tiered_canary_summary("code-745", "label-627");
        if item.move_to(TieredCanarySummaryStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredCanarySummaryStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_canary_summary("code-745", "label-627");
        item.status = TieredCanarySummaryStatus::Retired;
        assert!(
            item.move_to(TieredCanarySummaryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
