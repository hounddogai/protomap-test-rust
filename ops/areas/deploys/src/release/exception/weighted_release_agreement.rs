use super::*;

use acme_kit::text;

/// Where a weighted release agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedReleaseAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedReleaseAgreementStatus {
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

impl std::fmt::Display for WeightedReleaseAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted release agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedReleaseAgreement {
    pub code: String,
    pub label: String,
    pub host: String,
    pub tags: Vec<String>,
    pub ratio: f64,
    pub principal: String,
    pub ends_at: i64,
    pub category: String,
    pub status: WeightedReleaseAgreementStatus,
}

impl WeightedReleaseAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedReleaseAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedReleaseAgreementStatus) -> bool {
        match self.status {
            WeightedReleaseAgreementStatus::New => {
                match next {
                    WeightedReleaseAgreementStatus::Running | WeightedReleaseAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedReleaseAgreementStatus::Queued => {
                match next {
                    WeightedReleaseAgreementStatus::Running | WeightedReleaseAgreementStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedReleaseAgreementStatus::Running => {
                match next {
                    WeightedReleaseAgreementStatus::Done | WeightedReleaseAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedReleaseAgreementStatus::Done => {
                return next == WeightedReleaseAgreementStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedReleaseAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted release agreement {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "weighted release agreement", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.ends_at as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedReleaseAgreement {
        WeightedReleaseAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            tags: self.tags.clone(),
            ratio: self.ratio,
            principal: self.principal.clone(),
            ends_at: self.ends_at,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a weighted release agreement in status new.
pub fn new_weighted_release_agreement(code: &str, label: &str) -> WeightedReleaseAgreement {
    WeightedReleaseAgreement {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        tags: Vec::new(),
        ratio: 0.0,
        principal: "".to_string(),
        ends_at: 0,
        category: "".to_string(),
        status: WeightedReleaseAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_weighted_release_agreement("code-255", "label-174");
        assert_eq!(item.status, WeightedReleaseAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_weighted_release_agreement("code-255", "label-174");
        if item.move_to(WeightedReleaseAgreementStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedReleaseAgreementStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_weighted_release_agreement("code-255", "label-174");
        item.status = WeightedReleaseAgreementStatus::Failed;
        assert!(
            item.move_to(WeightedReleaseAgreementStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
