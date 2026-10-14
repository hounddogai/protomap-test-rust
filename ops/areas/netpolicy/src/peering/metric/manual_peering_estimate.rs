use super::*;

use acme_kit::text;

/// Where a manual peering estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualPeeringEstimateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualPeeringEstimateStatus {
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

impl std::fmt::Display for ManualPeeringEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual peering estimate as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringEstimate {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub reviewer: String,
    pub port: i64,
    pub burst: i64,
    pub archived: bool,
    pub status: ManualPeeringEstimateStatus,
}

impl ManualPeeringEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualPeeringEstimateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualPeeringEstimateStatus) -> bool {
        match self.status {
            ManualPeeringEstimateStatus::New => {
                return next == ManualPeeringEstimateStatus::Failed;
            }
            ManualPeeringEstimateStatus::Queued => {
                return next == ManualPeeringEstimateStatus::Failed;
            }
            ManualPeeringEstimateStatus::Running => {
                match next {
                    ManualPeeringEstimateStatus::Done | ManualPeeringEstimateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualPeeringEstimateStatus::Done => {
                match next {
                    ManualPeeringEstimateStatus::Failed => {
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

    pub fn move_to(&mut self, next: ManualPeeringEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual peering estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.burst as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualPeeringEstimate {
        ManualPeeringEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            reviewer: self.reviewer.clone(),
            port: self.port,
            burst: self.burst,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual peering estimate in status new.
pub fn new_manual_peering_estimate(code: &str, title: &str) -> ManualPeeringEstimate {
    ManualPeeringEstimate {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        reviewer: "".to_string(),
        port: 0,
        burst: 0,
        archived: false,
        status: ManualPeeringEstimateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_peering_estimate("code-68", "title-194");
        assert_eq!(item.status, ManualPeeringEstimateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_manual_peering_estimate("code-68", "title-194");
        if item.move_to(ManualPeeringEstimateStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualPeeringEstimateStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_peering_estimate("code-68", "title-194");
        item.status = ManualPeeringEstimateStatus::Failed;
        assert!(
            item.move_to(ManualPeeringEstimateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
