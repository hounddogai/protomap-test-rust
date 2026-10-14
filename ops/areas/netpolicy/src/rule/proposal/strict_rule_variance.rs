use super::*;

use acme_kit::text;

/// Where a strict rule variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRuleVarianceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictRuleVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for StrictRuleVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict rule variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleVariance {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub tags: Vec<String>,
    pub locale: String,
    pub timezone: String,
    pub rate: f64,
    pub active: bool,
    pub external_ref: String,
    pub threshold: f64,
    pub channel: String,
    pub status: StrictRuleVarianceStatus,
}

impl StrictRuleVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRuleVarianceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRuleVarianceStatus) -> bool {
        match self.status {
            StrictRuleVarianceStatus::Open => {
                return next == StrictRuleVarianceStatus::Archived;
            }
            StrictRuleVarianceStatus::InReview => {
                return next == StrictRuleVarianceStatus::Resolved || next == StrictRuleVarianceStatus::Reopened;
            }
            StrictRuleVarianceStatus::Resolved => {
                match next {
                    StrictRuleVarianceStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRuleVarianceStatus::Reopened => {
                match next {
                    StrictRuleVarianceStatus::Archived => {
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

    pub fn move_to(&mut self, next: StrictRuleVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict rule variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict rule variance", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.discount * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictRuleVariance {
        StrictRuleVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            tags: self.tags.clone(),
            locale: self.locale.clone(),
            timezone: self.timezone.clone(),
            rate: self.rate,
            active: self.active,
            external_ref: self.external_ref.clone(),
            threshold: self.threshold,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a strict rule variance in status open.
pub fn new_strict_rule_variance(code: &str, title: &str) -> StrictRuleVariance {
    StrictRuleVariance {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        tags: Vec::new(),
        locale: "".to_string(),
        timezone: "".to_string(),
        rate: 0.0,
        active: false,
        external_ref: "".to_string(),
        threshold: 0.0,
        channel: "".to_string(),
        status: StrictRuleVarianceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_rule_variance("code-207", "title-803");
        assert_eq!(item.status, StrictRuleVarianceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_strict_rule_variance("code-207", "title-803");
        if item.move_to(StrictRuleVarianceStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRuleVarianceStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_rule_variance("code-207", "title-803");
        item.status = StrictRuleVarianceStatus::Archived;
        assert!(
            item.move_to(StrictRuleVarianceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
