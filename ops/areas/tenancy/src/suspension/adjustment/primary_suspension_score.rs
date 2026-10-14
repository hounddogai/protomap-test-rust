use super::*;

use acme_kit::text;

/// Where a primary suspension score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySuspensionScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimarySuspensionScoreStatus {
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

impl std::fmt::Display for PrimarySuspensionScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary suspension score as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionScore {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub active: bool,
    pub amount: f64,
    pub starts_at: i64,
    pub audience: String,
    pub margin: f64,
    pub contact_email: String,
    pub cluster: String,
    pub limit: i64,
    pub status: PrimarySuspensionScoreStatus,
}

impl PrimarySuspensionScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySuspensionScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySuspensionScoreStatus) -> bool {
        match self.status {
            PrimarySuspensionScoreStatus::New => {
                match next {
                    PrimarySuspensionScoreStatus::Running | PrimarySuspensionScoreStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySuspensionScoreStatus::Queued => {
                match next {
                    PrimarySuspensionScoreStatus::Running | PrimarySuspensionScoreStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySuspensionScoreStatus::Running => {
                return next == PrimarySuspensionScoreStatus::Done;
            }
            PrimarySuspensionScoreStatus::Done => {
                return next == PrimarySuspensionScoreStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimarySuspensionScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary suspension score {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.limit as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySuspensionScore {
        PrimarySuspensionScore {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            active: self.active,
            amount: self.amount,
            starts_at: self.starts_at,
            audience: self.audience.clone(),
            margin: self.margin,
            contact_email: self.contact_email.clone(),
            cluster: self.cluster.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary suspension score in status new.
pub fn new_primary_suspension_score(code: &str, label: &str) -> PrimarySuspensionScore {
    PrimarySuspensionScore {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        active: false,
        amount: 0.0,
        starts_at: 0,
        audience: "".to_string(),
        margin: 0.0,
        contact_email: "".to_string(),
        cluster: "".to_string(),
        limit: 0,
        status: PrimarySuspensionScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_suspension_score("code-287", "label-842");
        assert_eq!(item.status, PrimarySuspensionScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_primary_suspension_score("code-287", "label-842");
        if item.move_to(PrimarySuspensionScoreStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySuspensionScoreStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_suspension_score("code-287", "label-842");
        item.status = PrimarySuspensionScoreStatus::Failed;
        assert!(
            item.move_to(PrimarySuspensionScoreStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
