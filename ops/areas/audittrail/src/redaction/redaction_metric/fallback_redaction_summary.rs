use super::*;

use acme_kit::text;

/// Where a fallback redaction summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRedactionSummaryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackRedactionSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackRedactionSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback redaction summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionSummary {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub burst: i64,
    pub timezone: String,
    pub cost: f64,
    pub reviewer: String,
    pub owner_id: String,
    pub version: i64,
    pub status: FallbackRedactionSummaryStatus,
}

impl FallbackRedactionSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRedactionSummaryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRedactionSummaryStatus) -> bool {
        match self.status {
            FallbackRedactionSummaryStatus::Created => {
                match next {
                    FallbackRedactionSummaryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRedactionSummaryStatus::Validated => {
                return next == FallbackRedactionSummaryStatus::Published
                    || next == FallbackRedactionSummaryStatus::Retired;
            }
            FallbackRedactionSummaryStatus::Published => {
                match next {
                    FallbackRedactionSummaryStatus::Retired => {
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

    pub fn move_to(&mut self, next: FallbackRedactionSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback redaction summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback redaction summary", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.burst as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRedactionSummary {
        FallbackRedactionSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            burst: self.burst,
            timezone: self.timezone.clone(),
            cost: self.cost,
            reviewer: self.reviewer.clone(),
            owner_id: self.owner_id.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback redaction summary in status created.
pub fn new_fallback_redaction_summary(code: &str, title: &str) -> FallbackRedactionSummary {
    FallbackRedactionSummary {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        burst: 0,
        timezone: "".to_string(),
        cost: 0.0,
        reviewer: "".to_string(),
        owner_id: "".to_string(),
        version: 0,
        status: FallbackRedactionSummaryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_redaction_summary("code-776", "title-330");
        assert_eq!(item.status, FallbackRedactionSummaryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_fallback_redaction_summary("code-776", "title-330");
        if item.move_to(FallbackRedactionSummaryStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRedactionSummaryStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_redaction_summary("code-776", "title-330");
        item.status = FallbackRedactionSummaryStatus::Retired;
        assert!(
            item.move_to(FallbackRedactionSummaryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
