use super::*;

use acme_kit::text;

/// Where a evidence schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceScheduleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EvidenceScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EvidenceScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSchedule {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub title: String,
    pub retries: i64,
    pub cluster: String,
    pub country: String,
    pub approved: bool,
    pub rate: f64,
    pub status: EvidenceScheduleStatus,
}

impl EvidenceSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceScheduleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceScheduleStatus) -> bool {
        match self.status {
            EvidenceScheduleStatus::Created => {
                match next {
                    EvidenceScheduleStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceScheduleStatus::Validated => {
                return next == EvidenceScheduleStatus::Published;
            }
            EvidenceScheduleStatus::Published => {
                return next == EvidenceScheduleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "evidence schedule", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.retries as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceSchedule {
        EvidenceSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            title: self.title.clone(),
            retries: self.retries,
            cluster: self.cluster.clone(),
            country: self.country.clone(),
            approved: self.approved,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a evidence schedule in status created.
pub fn new_evidence_schedule(code: &str, label: &str) -> EvidenceSchedule {
    EvidenceSchedule {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        title: "".to_string(),
        retries: 0,
        cluster: "".to_string(),
        country: "".to_string(),
        approved: false,
        rate: 0.0,
        status: EvidenceScheduleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_evidence_schedule("code-699", "label-602");
        assert_eq!(item.status, EvidenceScheduleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_evidence_schedule("code-699", "label-602");
        if item.move_to(EvidenceScheduleStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceScheduleStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_evidence_schedule("code-699", "label-602");
        item.status = EvidenceScheduleStatus::Retired;
        assert!(
            item.move_to(EvidenceScheduleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
