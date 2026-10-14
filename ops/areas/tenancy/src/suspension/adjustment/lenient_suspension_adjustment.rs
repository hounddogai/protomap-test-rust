use super::*;

use acme_kit::text;

/// Where a lenient suspension adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSuspensionAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientSuspensionAdjustmentStatus {
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

impl std::fmt::Display for LenientSuspensionAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient suspension adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionAdjustment {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub ends_at: i64,
    pub visible: bool,
    pub source: String,
    pub minimum: f64,
    pub quota_units: i64,
    pub ratio: f64,
    pub phone: String,
    pub status: LenientSuspensionAdjustmentStatus,
}

impl LenientSuspensionAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSuspensionAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSuspensionAdjustmentStatus) -> bool {
        match self.status {
            LenientSuspensionAdjustmentStatus::New => {
                match next {
                    LenientSuspensionAdjustmentStatus::Queued | LenientSuspensionAdjustmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSuspensionAdjustmentStatus::Queued => {
                match next {
                    LenientSuspensionAdjustmentStatus::Done | LenientSuspensionAdjustmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSuspensionAdjustmentStatus::Running => {
                match next {
                    LenientSuspensionAdjustmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSuspensionAdjustmentStatus::Done => {
                return next == LenientSuspensionAdjustmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientSuspensionAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient suspension adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ends_at as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSuspensionAdjustment {
        LenientSuspensionAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            ends_at: self.ends_at,
            visible: self.visible,
            source: self.source.clone(),
            minimum: self.minimum,
            quota_units: self.quota_units,
            ratio: self.ratio,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a lenient suspension adjustment in status new.
pub fn new_lenient_suspension_adjustment(code: &str, title: &str) -> LenientSuspensionAdjustment {
    LenientSuspensionAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        ends_at: 0,
        visible: false,
        source: "".to_string(),
        minimum: 0.0,
        quota_units: 0,
        ratio: 0.0,
        phone: "".to_string(),
        status: LenientSuspensionAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_suspension_adjustment("code-389", "title-517");
        assert_eq!(item.status, LenientSuspensionAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_lenient_suspension_adjustment("code-389", "title-517");
        if item.move_to(LenientSuspensionAdjustmentStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSuspensionAdjustmentStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_suspension_adjustment("code-389", "title-517");
        item.status = LenientSuspensionAdjustmentStatus::Failed;
        assert!(
            item.move_to(LenientSuspensionAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
