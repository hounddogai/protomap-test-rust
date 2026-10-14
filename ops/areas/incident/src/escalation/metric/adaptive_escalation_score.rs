use super::*;

use acme_kit::text;

/// Where a adaptive escalation score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveEscalationScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveEscalationScoreStatus {
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

impl std::fmt::Display for AdaptiveEscalationScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive escalation score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEscalationScore {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub amount: f64,
    pub label: String,
    pub locale: String,
    pub rate: f64,
    pub status: AdaptiveEscalationScoreStatus,
}

impl AdaptiveEscalationScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveEscalationScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveEscalationScoreStatus) -> bool {
        match self.status {
            AdaptiveEscalationScoreStatus::Open => {
                return next == AdaptiveEscalationScoreStatus::InReview
                    || next == AdaptiveEscalationScoreStatus::Reopened;
            }
            AdaptiveEscalationScoreStatus::InReview => {
                match next {
                    AdaptiveEscalationScoreStatus::Resolved | AdaptiveEscalationScoreStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveEscalationScoreStatus::Resolved => {
                return next == AdaptiveEscalationScoreStatus::Reopened
                    || next == AdaptiveEscalationScoreStatus::Archived;
            }
            AdaptiveEscalationScoreStatus::Reopened => {
                match next {
                    AdaptiveEscalationScoreStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveEscalationScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive escalation score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.amount + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveEscalationScore {
        AdaptiveEscalationScore {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            amount: self.amount,
            label: self.label.clone(),
            locale: self.locale.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a adaptive escalation score in status open.
pub fn new_adaptive_escalation_score(code: &str, title: &str) -> AdaptiveEscalationScore {
    AdaptiveEscalationScore {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        amount: 0.0,
        label: "".to_string(),
        locale: "".to_string(),
        rate: 0.0,
        status: AdaptiveEscalationScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_escalation_score("code-955", "title-447");
        assert_eq!(item.status, AdaptiveEscalationScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_adaptive_escalation_score("code-955", "title-447");
        if item.move_to(AdaptiveEscalationScoreStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveEscalationScoreStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_escalation_score("code-955", "title-447");
        item.status = AdaptiveEscalationScoreStatus::Archived;
        assert!(
            item.move_to(AdaptiveEscalationScoreStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
