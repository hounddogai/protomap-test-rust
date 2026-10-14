use super::*;

use acme_kit::text;

/// Where a capped postmortem forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedPostmortemForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedPostmortemForecastStatus {
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

impl std::fmt::Display for CappedPostmortemForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped postmortem forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemForecast {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub quantity: i64,
    pub threshold: f64,
    pub namespace: String,
    pub discount: f64,
    pub minimum: f64,
    pub status: CappedPostmortemForecastStatus,
}

impl CappedPostmortemForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedPostmortemForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedPostmortemForecastStatus) -> bool {
        match self.status {
            CappedPostmortemForecastStatus::New => {
                match next {
                    CappedPostmortemForecastStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedPostmortemForecastStatus::Queued => {
                return next == CappedPostmortemForecastStatus::Done || next == CappedPostmortemForecastStatus::Failed;
            }
            CappedPostmortemForecastStatus::Running => {
                return next == CappedPostmortemForecastStatus::Done || next == CappedPostmortemForecastStatus::Failed;
            }
            CappedPostmortemForecastStatus::Done => {
                match next {
                    CappedPostmortemForecastStatus::Failed => {
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

    pub fn move_to(&mut self, next: CappedPostmortemForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped postmortem forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        (self.threshold * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedPostmortemForecast {
        CappedPostmortemForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            quantity: self.quantity,
            threshold: self.threshold,
            namespace: self.namespace.clone(),
            discount: self.discount,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a capped postmortem forecast in status new.
pub fn new_capped_postmortem_forecast(code: &str, label: &str) -> CappedPostmortemForecast {
    CappedPostmortemForecast {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        quantity: 0,
        threshold: 0.0,
        namespace: "".to_string(),
        discount: 0.0,
        minimum: 0.0,
        status: CappedPostmortemForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_postmortem_forecast("code-448", "label-727");
        assert_eq!(item.status, CappedPostmortemForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_capped_postmortem_forecast("code-448", "label-727");
        if item.move_to(CappedPostmortemForecastStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedPostmortemForecastStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_postmortem_forecast("code-448", "label-727");
        item.status = CappedPostmortemForecastStatus::Failed;
        assert!(
            item.move_to(CappedPostmortemForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
