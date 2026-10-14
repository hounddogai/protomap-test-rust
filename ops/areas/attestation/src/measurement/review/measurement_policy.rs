use super::*;

use acme_kit::text;

/// Where a measurement policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementPolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl MeasurementPolicyStatus {
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

impl std::fmt::Display for MeasurementPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a measurement policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPolicy {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub contact_email: String,
    pub owner_id: String,
    pub score: f64,
    pub status: MeasurementPolicyStatus,
}

impl MeasurementPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementPolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementPolicyStatus) -> bool {
        match self.status {
            MeasurementPolicyStatus::New => {
                return next == MeasurementPolicyStatus::Queued || next == MeasurementPolicyStatus::Failed;
            }
            MeasurementPolicyStatus::Queued => {
                return next == MeasurementPolicyStatus::Done || next == MeasurementPolicyStatus::Failed;
            }
            MeasurementPolicyStatus::Running => {
                return next == MeasurementPolicyStatus::Done || next == MeasurementPolicyStatus::Failed;
            }
            MeasurementPolicyStatus::Done => {
                match next {
                    MeasurementPolicyStatus::Failed => {
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

    pub fn move_to(&mut self, next: MeasurementPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> MeasurementPolicy {
        MeasurementPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            contact_email: self.contact_email.clone(),
            owner_id: self.owner_id.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a measurement policy in status new.
pub fn new_measurement_policy(code: &str, title: &str) -> MeasurementPolicy {
    MeasurementPolicy {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        contact_email: "".to_string(),
        owner_id: "".to_string(),
        score: 0.0,
        status: MeasurementPolicyStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_measurement_policy("code-470", "title-569");
        assert_eq!(item.status, MeasurementPolicyStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_measurement_policy("code-470", "title-569");
        if item.move_to(MeasurementPolicyStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementPolicyStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_measurement_policy("code-470", "title-569");
        item.status = MeasurementPolicyStatus::Failed;
        assert!(
            item.move_to(MeasurementPolicyStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
