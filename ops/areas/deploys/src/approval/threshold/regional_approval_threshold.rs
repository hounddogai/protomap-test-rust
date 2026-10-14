use super::*;

use acme_kit::text;

/// Where a regional approval threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalApprovalThresholdStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalApprovalThresholdStatus {
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

impl std::fmt::Display for RegionalApprovalThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional approval threshold as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalThreshold {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub category: String,
    pub burst: i64,
    pub issuer: String,
    pub version: i64,
    pub budget: f64,
    pub margin: f64,
    pub status: RegionalApprovalThresholdStatus,
}

impl RegionalApprovalThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalApprovalThresholdStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalApprovalThresholdStatus) -> bool {
        match self.status {
            RegionalApprovalThresholdStatus::New => {
                match next {
                    RegionalApprovalThresholdStatus::Queued | RegionalApprovalThresholdStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalApprovalThresholdStatus::Queued => {
                match next {
                    RegionalApprovalThresholdStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalApprovalThresholdStatus::Running => {
                return next == RegionalApprovalThresholdStatus::Done;
            }
            RegionalApprovalThresholdStatus::Done => {
                return next == RegionalApprovalThresholdStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalApprovalThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional approval threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.budget + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalApprovalThreshold {
        RegionalApprovalThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            category: self.category.clone(),
            burst: self.burst,
            issuer: self.issuer.clone(),
            version: self.version,
            budget: self.budget,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a regional approval threshold in status new.
pub fn new_regional_approval_threshold(code: &str, title: &str) -> RegionalApprovalThreshold {
    RegionalApprovalThreshold {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        category: "".to_string(),
        burst: 0,
        issuer: "".to_string(),
        version: 0,
        budget: 0.0,
        margin: 0.0,
        status: RegionalApprovalThresholdStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_approval_threshold("code-263", "title-319");
        assert_eq!(item.status, RegionalApprovalThresholdStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_regional_approval_threshold("code-263", "title-319");
        if item.move_to(RegionalApprovalThresholdStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalApprovalThresholdStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_approval_threshold("code-263", "title-319");
        item.status = RegionalApprovalThresholdStatus::Failed;
        assert!(
            item.move_to(RegionalApprovalThresholdStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
