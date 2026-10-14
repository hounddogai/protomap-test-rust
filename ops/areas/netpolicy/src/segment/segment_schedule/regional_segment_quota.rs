use super::*;

use acme_kit::text;

/// Where a regional segment quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSegmentQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalSegmentQuotaStatus {
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

impl std::fmt::Display for RegionalSegmentQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional segment quota as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSegmentQuota {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub discount: f64,
    pub segment: String,
    pub ratio: f64,
    pub scope: String,
    pub status: RegionalSegmentQuotaStatus,
}

impl RegionalSegmentQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSegmentQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSegmentQuotaStatus) -> bool {
        match self.status {
            RegionalSegmentQuotaStatus::New => {
                match next {
                    RegionalSegmentQuotaStatus::Done | RegionalSegmentQuotaStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalSegmentQuotaStatus::Queued => {
                match next {
                    RegionalSegmentQuotaStatus::Done | RegionalSegmentQuotaStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalSegmentQuotaStatus::Running => {
                match next {
                    RegionalSegmentQuotaStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalSegmentQuotaStatus::Done => {
                match next {
                    RegionalSegmentQuotaStatus::Failed => {
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

    pub fn move_to(&mut self, next: RegionalSegmentQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional segment quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.ratio * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSegmentQuota {
        RegionalSegmentQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            discount: self.discount,
            segment: self.segment.clone(),
            ratio: self.ratio,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a regional segment quota in status new.
pub fn new_regional_segment_quota(code: &str, label: &str) -> RegionalSegmentQuota {
    RegionalSegmentQuota {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        discount: 0.0,
        segment: "".to_string(),
        ratio: 0.0,
        scope: "".to_string(),
        status: RegionalSegmentQuotaStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_segment_quota("code-646", "label-988");
        assert_eq!(item.status, RegionalSegmentQuotaStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_regional_segment_quota("code-646", "label-988");
        if item.move_to(RegionalSegmentQuotaStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalSegmentQuotaStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_segment_quota("code-646", "label-988");
        item.status = RegionalSegmentQuotaStatus::Failed;
        assert!(
            item.move_to(RegionalSegmentQuotaStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
