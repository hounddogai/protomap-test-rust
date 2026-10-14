use super::*;

use acme_kit::text;

/// Where a quote variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl QuoteVarianceStatus {
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

impl std::fmt::Display for QuoteVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote variance as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteVariance {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub visible: bool,
    pub weight: f64,
    pub label: String,
    pub status: QuoteVarianceStatus,
}

impl QuoteVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteVarianceStatus) -> bool {
        match self.status {
            QuoteVarianceStatus::New => {
                return next == QuoteVarianceStatus::Queued || next == QuoteVarianceStatus::Done;
            }
            QuoteVarianceStatus::Queued => {
                return next == QuoteVarianceStatus::Running || next == QuoteVarianceStatus::Done;
            }
            QuoteVarianceStatus::Running => {
                return next == QuoteVarianceStatus::Done;
            }
            QuoteVarianceStatus::Done => {
                return next == QuoteVarianceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> QuoteVariance {
        QuoteVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            visible: self.visible,
            weight: self.weight,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a quote variance in status new.
pub fn new_quote_variance(code: &str, name: &str) -> QuoteVariance {
    QuoteVariance {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        visible: false,
        weight: 0.0,
        label: "".to_string(),
        status: QuoteVarianceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_quote_variance("code-477", "name-608");
        assert_eq!(item.status, QuoteVarianceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_quote_variance("code-477", "name-608");
        if item.move_to(QuoteVarianceStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteVarianceStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_quote_variance("code-477", "name-608");
        item.status = QuoteVarianceStatus::Failed;
        assert!(item.move_to(QuoteVarianceStatus::New).is_err(), "expected move to fail");
    }
}
