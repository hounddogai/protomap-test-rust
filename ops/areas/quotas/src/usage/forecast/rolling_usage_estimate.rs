use super::*;

use acme_kit::text;

/// Where a rolling usage estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingUsageEstimateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingUsageEstimateStatus {
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

impl std::fmt::Display for RollingUsageEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling usage estimate as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageEstimate {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub reviewer: String,
    pub host: String,
    pub region: String,
    pub contact_email: String,
    pub ratio: f64,
    pub version: i64,
    pub fingerprint: String,
    pub notes: String,
    pub status: RollingUsageEstimateStatus,
}

impl RollingUsageEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingUsageEstimateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingUsageEstimateStatus) -> bool {
        match self.status {
            RollingUsageEstimateStatus::New => {
                match next {
                    RollingUsageEstimateStatus::Done | RollingUsageEstimateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingUsageEstimateStatus::Queued => {
                return next == RollingUsageEstimateStatus::Failed;
            }
            RollingUsageEstimateStatus::Running => {
                match next {
                    RollingUsageEstimateStatus::Done | RollingUsageEstimateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingUsageEstimateStatus::Done => {
                match next {
                    RollingUsageEstimateStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingUsageEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling usage estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling usage estimate", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.version as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingUsageEstimate {
        RollingUsageEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            reviewer: self.reviewer.clone(),
            host: self.host.clone(),
            region: self.region.clone(),
            contact_email: self.contact_email.clone(),
            ratio: self.ratio,
            version: self.version,
            fingerprint: self.fingerprint.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rolling usage estimate in status new.
pub fn new_rolling_usage_estimate(code: &str, label: &str) -> RollingUsageEstimate {
    RollingUsageEstimate {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        reviewer: "".to_string(),
        host: "".to_string(),
        region: "".to_string(),
        contact_email: "".to_string(),
        ratio: 0.0,
        version: 0,
        fingerprint: "".to_string(),
        notes: "".to_string(),
        status: RollingUsageEstimateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_usage_estimate("code-142", "label-465");
        assert_eq!(item.status, RollingUsageEstimateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_rolling_usage_estimate("code-142", "label-465");
        if item.move_to(RollingUsageEstimateStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingUsageEstimateStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_usage_estimate("code-142", "label-465");
        item.status = RollingUsageEstimateStatus::Failed;
        assert!(
            item.move_to(RollingUsageEstimateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
