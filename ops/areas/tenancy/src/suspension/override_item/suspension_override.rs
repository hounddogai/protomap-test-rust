use super::*;

use acme_kit::text;

/// Where a suspension override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SuspensionOverrideStatus {
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

impl std::fmt::Display for SuspensionOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One suspension override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionOverride {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub weight: f64,
    pub created_at: i64,
    pub issuer: String,
    pub maximum: f64,
    pub threshold: f64,
    pub phone: String,
    pub status: SuspensionOverrideStatus,
}

impl SuspensionOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionOverrideStatus) -> bool {
        match self.status {
            SuspensionOverrideStatus::New => {
                return next == SuspensionOverrideStatus::Queued || next == SuspensionOverrideStatus::Running;
            }
            SuspensionOverrideStatus::Queued => {
                return next == SuspensionOverrideStatus::Done;
            }
            SuspensionOverrideStatus::Running => {
                return next == SuspensionOverrideStatus::Done || next == SuspensionOverrideStatus::Failed;
            }
            SuspensionOverrideStatus::Done => {
                match next {
                    SuspensionOverrideStatus::Failed => {
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

    pub fn move_to(&mut self, next: SuspensionOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.created_at as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionOverride {
        SuspensionOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            weight: self.weight,
            created_at: self.created_at,
            issuer: self.issuer.clone(),
            maximum: self.maximum,
            threshold: self.threshold,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a suspension override in status new.
pub fn new_suspension_override(code: &str, title: &str) -> SuspensionOverride {
    SuspensionOverride {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        weight: 0.0,
        created_at: 0,
        issuer: "".to_string(),
        maximum: 0.0,
        threshold: 0.0,
        phone: "".to_string(),
        status: SuspensionOverrideStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_suspension_override("code-756", "title-382");
        assert_eq!(item.status, SuspensionOverrideStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_suspension_override("code-756", "title-382");
        if item.move_to(SuspensionOverrideStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionOverrideStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_suspension_override("code-756", "title-382");
        item.status = SuspensionOverrideStatus::Failed;
        assert!(
            item.move_to(SuspensionOverrideStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
