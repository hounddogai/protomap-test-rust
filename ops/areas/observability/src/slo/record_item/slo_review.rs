use super::*;

use acme_kit::text;

/// Where a slo review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SloReviewStatus {
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

impl std::fmt::Display for SloReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo review as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloReview {
    pub code: String,
    pub name: String,
    pub country: String,
    pub checksum: String,
    pub weight: f64,
    pub source: String,
    pub label: String,
    pub discount: f64,
    pub sequence: i64,
    pub host: String,
    pub summary: String,
    pub status: SloReviewStatus,
}

impl SloReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloReviewStatus) -> bool {
        match self.status {
            SloReviewStatus::New => {
                return next == SloReviewStatus::Done;
            }
            SloReviewStatus::Queued => {
                match next {
                    SloReviewStatus::Running | SloReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloReviewStatus::Running => {
                match next {
                    SloReviewStatus::Done | SloReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloReviewStatus::Done => {
                match next {
                    SloReviewStatus::Failed => {
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

    pub fn move_to(&mut self, next: SloReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.discount * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloReview {
        SloReview {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            checksum: self.checksum.clone(),
            weight: self.weight,
            source: self.source.clone(),
            label: self.label.clone(),
            discount: self.discount,
            sequence: self.sequence,
            host: self.host.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a slo review in status new.
pub fn new_slo_review(code: &str, name: &str) -> SloReview {
    SloReview {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        checksum: "".to_string(),
        weight: 0.0,
        source: "".to_string(),
        label: "".to_string(),
        discount: 0.0,
        sequence: 0,
        host: "".to_string(),
        summary: "".to_string(),
        status: SloReviewStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_slo_review("code-583", "name-644");
        assert_eq!(item.status, SloReviewStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_slo_review("code-583", "name-644");
        if item.move_to(SloReviewStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloReviewStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_slo_review("code-583", "name-644");
        item.status = SloReviewStatus::Failed;
        assert!(item.move_to(SloReviewStatus::New).is_err(), "expected move to fail");
    }
}
