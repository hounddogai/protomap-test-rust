use super::*;

use acme_kit::text;

/// Where a lenient probe group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientProbeGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientProbeGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LenientProbeGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient probe group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeGroup {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub capacity: i64,
    pub ratio: f64,
    pub updated_at: i64,
    pub principal: String,
    pub active: bool,
    pub status: LenientProbeGroupStatus,
}

impl LenientProbeGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientProbeGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientProbeGroupStatus) -> bool {
        match self.status {
            LenientProbeGroupStatus::Planned => {
                match next {
                    LenientProbeGroupStatus::Scheduled | LenientProbeGroupStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientProbeGroupStatus::Scheduled => {
                return next == LenientProbeGroupStatus::Completed || next == LenientProbeGroupStatus::Cancelled;
            }
            LenientProbeGroupStatus::InProgress => {
                return next == LenientProbeGroupStatus::Completed;
            }
            LenientProbeGroupStatus::Completed => {
                match next {
                    LenientProbeGroupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientProbeGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient probe group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient probe group", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.updated_at as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientProbeGroup {
        LenientProbeGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            capacity: self.capacity,
            ratio: self.ratio,
            updated_at: self.updated_at,
            principal: self.principal.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient probe group in status planned.
pub fn new_lenient_probe_group(code: &str, title: &str) -> LenientProbeGroup {
    LenientProbeGroup {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        capacity: 0,
        ratio: 0.0,
        updated_at: 0,
        principal: "".to_string(),
        active: false,
        status: LenientProbeGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_probe_group("code-324", "title-161");
        assert_eq!(item.status, LenientProbeGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_lenient_probe_group("code-324", "title-161");
        if item.move_to(LenientProbeGroupStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientProbeGroupStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_probe_group("code-324", "title-161");
        item.status = LenientProbeGroupStatus::Cancelled;
        assert!(
            item.move_to(LenientProbeGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
