use super::*;

use acme_kit::text;

/// Where a suspension bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SuspensionBundleStatus {
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

impl std::fmt::Display for SuspensionBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One suspension bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBundle {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub timezone: String,
    pub batch_size: i64,
    pub audience: String,
    pub amount: f64,
    pub starts_at: i64,
    pub ratio: f64,
    pub status: SuspensionBundleStatus,
}

impl SuspensionBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionBundleStatus) -> bool {
        match self.status {
            SuspensionBundleStatus::New => {
                match next {
                    SuspensionBundleStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionBundleStatus::Queued => {
                return next == SuspensionBundleStatus::Failed;
            }
            SuspensionBundleStatus::Running => {
                match next {
                    SuspensionBundleStatus::Done | SuspensionBundleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionBundleStatus::Done => {
                return next == SuspensionBundleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension bundle {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.batch_size as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionBundle {
        SuspensionBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            timezone: self.timezone.clone(),
            batch_size: self.batch_size,
            audience: self.audience.clone(),
            amount: self.amount,
            starts_at: self.starts_at,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a suspension bundle in status new.
pub fn new_suspension_bundle(code: &str, label: &str) -> SuspensionBundle {
    SuspensionBundle {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        timezone: "".to_string(),
        batch_size: 0,
        audience: "".to_string(),
        amount: 0.0,
        starts_at: 0,
        ratio: 0.0,
        status: SuspensionBundleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_suspension_bundle("code-936", "label-162");
        assert_eq!(item.status, SuspensionBundleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_suspension_bundle("code-936", "label-162");
        if item.move_to(SuspensionBundleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionBundleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_suspension_bundle("code-936", "label-162");
        item.status = SuspensionBundleStatus::Failed;
        assert!(
            item.move_to(SuspensionBundleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
