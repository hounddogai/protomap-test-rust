use super::*;

use acme_kit::text;

/// Where a manual quote variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualQuoteVarianceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualQuoteVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ManualQuoteVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual quote variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteVariance {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub minimum: f64,
    pub grace_days: i64,
    pub summary: String,
    pub approved: bool,
    pub category: String,
    pub status: ManualQuoteVarianceStatus,
}

impl ManualQuoteVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualQuoteVarianceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualQuoteVarianceStatus) -> bool {
        match self.status {
            ManualQuoteVarianceStatus::Draft => {
                match next {
                    ManualQuoteVarianceStatus::Pending | ManualQuoteVarianceStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualQuoteVarianceStatus::Pending => {
                return next == ManualQuoteVarianceStatus::Suspended;
            }
            ManualQuoteVarianceStatus::Active => {
                return next == ManualQuoteVarianceStatus::Suspended;
            }
            ManualQuoteVarianceStatus::Suspended => {
                match next {
                    ManualQuoteVarianceStatus::Closed => {
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

    pub fn move_to(&mut self, next: ManualQuoteVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual quote variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual quote variance", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.version as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualQuoteVariance {
        ManualQuoteVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            minimum: self.minimum,
            grace_days: self.grace_days,
            summary: self.summary.clone(),
            approved: self.approved,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a manual quote variance in status draft.
pub fn new_manual_quote_variance(code: &str, name: &str) -> ManualQuoteVariance {
    ManualQuoteVariance {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        minimum: 0.0,
        grace_days: 0,
        summary: "".to_string(),
        approved: false,
        category: "".to_string(),
        status: ManualQuoteVarianceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_quote_variance("code-435", "name-976");
        assert_eq!(item.status, ManualQuoteVarianceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_manual_quote_variance("code-435", "name-976");
        if item.move_to(ManualQuoteVarianceStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualQuoteVarianceStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_quote_variance("code-435", "name-976");
        item.status = ManualQuoteVarianceStatus::Closed;
        assert!(
            item.move_to(ManualQuoteVarianceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
