use super::*;

use acme_kit::text;

/// Where a batched reservation template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedReservationTemplateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedReservationTemplateStatus {
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

impl std::fmt::Display for BatchedReservationTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched reservation template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationTemplate {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub archived: bool,
    pub channel: String,
    pub rate: f64,
    pub cluster: String,
    pub fingerprint: String,
    pub notes: String,
    pub status: BatchedReservationTemplateStatus,
}

impl BatchedReservationTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedReservationTemplateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedReservationTemplateStatus) -> bool {
        match self.status {
            BatchedReservationTemplateStatus::New => {
                match next {
                    BatchedReservationTemplateStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedReservationTemplateStatus::Queued => {
                match next {
                    BatchedReservationTemplateStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedReservationTemplateStatus::Running => {
                return next == BatchedReservationTemplateStatus::Failed;
            }
            BatchedReservationTemplateStatus::Done => {
                match next {
                    BatchedReservationTemplateStatus::Failed => {
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

    pub fn move_to(&mut self, next: BatchedReservationTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched reservation template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched reservation template", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.rate * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedReservationTemplate {
        BatchedReservationTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            archived: self.archived,
            channel: self.channel.clone(),
            rate: self.rate,
            cluster: self.cluster.clone(),
            fingerprint: self.fingerprint.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a batched reservation template in status new.
pub fn new_batched_reservation_template(code: &str, title: &str) -> BatchedReservationTemplate {
    BatchedReservationTemplate {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        archived: false,
        channel: "".to_string(),
        rate: 0.0,
        cluster: "".to_string(),
        fingerprint: "".to_string(),
        notes: "".to_string(),
        status: BatchedReservationTemplateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_reservation_template("code-570", "title-36");
        assert_eq!(item.status, BatchedReservationTemplateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_batched_reservation_template("code-570", "title-36");
        if item.move_to(BatchedReservationTemplateStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedReservationTemplateStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_reservation_template("code-570", "title-36");
        item.status = BatchedReservationTemplateStatus::Failed;
        assert!(
            item.move_to(BatchedReservationTemplateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
