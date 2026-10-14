use super::*;

use acme_kit::text;

/// Where a weighted rule variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRuleVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedRuleVarianceStatus {
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

impl std::fmt::Display for WeightedRuleVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted rule variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleVariance {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub currency: String,
    pub window_days: i64,
    pub updated_at: i64,
    pub phone: String,
    pub source: String,
    pub starts_at: i64,
    pub burst: i64,
    pub description: String,
    pub status: WeightedRuleVarianceStatus,
}

impl WeightedRuleVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRuleVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRuleVarianceStatus) -> bool {
        match self.status {
            WeightedRuleVarianceStatus::New => {
                match next {
                    WeightedRuleVarianceStatus::Queued | WeightedRuleVarianceStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRuleVarianceStatus::Queued => {
                return next == WeightedRuleVarianceStatus::Done;
            }
            WeightedRuleVarianceStatus::Running => {
                return next == WeightedRuleVarianceStatus::Done;
            }
            WeightedRuleVarianceStatus::Done => {
                match next {
                    WeightedRuleVarianceStatus::Failed => {
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

    pub fn move_to(&mut self, next: WeightedRuleVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted rule variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted rule variance", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.burst as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRuleVariance {
        WeightedRuleVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            currency: self.currency.clone(),
            window_days: self.window_days,
            updated_at: self.updated_at,
            phone: self.phone.clone(),
            source: self.source.clone(),
            starts_at: self.starts_at,
            burst: self.burst,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a weighted rule variance in status new.
pub fn new_weighted_rule_variance(code: &str, title: &str) -> WeightedRuleVariance {
    WeightedRuleVariance {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        currency: "".to_string(),
        window_days: 0,
        updated_at: 0,
        phone: "".to_string(),
        source: "".to_string(),
        starts_at: 0,
        burst: 0,
        description: "".to_string(),
        status: WeightedRuleVarianceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_weighted_rule_variance("code-104", "title-754");
        assert_eq!(item.status, WeightedRuleVarianceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_weighted_rule_variance("code-104", "title-754");
        if item.move_to(WeightedRuleVarianceStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRuleVarianceStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_weighted_rule_variance("code-104", "title-754");
        item.status = WeightedRuleVarianceStatus::Failed;
        assert!(
            item.move_to(WeightedRuleVarianceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
