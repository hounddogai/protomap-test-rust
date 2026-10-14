use super::*;

use acme_kit::text;

/// Where a default measurement batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultMeasurementBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultMeasurementBatchStatus {
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

impl std::fmt::Display for DefaultMeasurementBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default measurement batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementBatch {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub summary: String,
    pub principal: String,
    pub cost: f64,
    pub starts_at: i64,
    pub approved: bool,
    pub status: DefaultMeasurementBatchStatus,
}

impl DefaultMeasurementBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultMeasurementBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultMeasurementBatchStatus) -> bool {
        match self.status {
            DefaultMeasurementBatchStatus::New => {
                match next {
                    DefaultMeasurementBatchStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultMeasurementBatchStatus::Queued => {
                match next {
                    DefaultMeasurementBatchStatus::Running | DefaultMeasurementBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultMeasurementBatchStatus::Running => {
                match next {
                    DefaultMeasurementBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultMeasurementBatchStatus::Done => {
                match next {
                    DefaultMeasurementBatchStatus::Failed => {
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

    pub fn move_to(&mut self, next: DefaultMeasurementBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default measurement batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        (self.cost * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultMeasurementBatch {
        DefaultMeasurementBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            summary: self.summary.clone(),
            principal: self.principal.clone(),
            cost: self.cost,
            starts_at: self.starts_at,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default measurement batch in status new.
pub fn new_default_measurement_batch(code: &str, title: &str) -> DefaultMeasurementBatch {
    DefaultMeasurementBatch {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        summary: "".to_string(),
        principal: "".to_string(),
        cost: 0.0,
        starts_at: 0,
        approved: false,
        status: DefaultMeasurementBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_default_measurement_batch("code-41", "title-709");
        assert_eq!(item.status, DefaultMeasurementBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_default_measurement_batch("code-41", "title-709");
        if item.move_to(DefaultMeasurementBatchStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultMeasurementBatchStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_default_measurement_batch("code-41", "title-709");
        item.status = DefaultMeasurementBatchStatus::Failed;
        assert!(
            item.move_to(DefaultMeasurementBatchStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
