use super::*;

use acme_kit::text;

/// Where a lenient grant variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientGrantVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientGrantVarianceStatus {
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

impl std::fmt::Display for LenientGrantVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient grant variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantVariance {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub checksum: String,
    pub grace_days: i64,
    pub weight: f64,
    pub channel: String,
    pub status: LenientGrantVarianceStatus,
}

impl LenientGrantVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientGrantVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientGrantVarianceStatus) -> bool {
        match self.status {
            LenientGrantVarianceStatus::Planned => {
                match next {
                    LenientGrantVarianceStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientGrantVarianceStatus::Scheduled => {
                return next == LenientGrantVarianceStatus::InProgress || next == LenientGrantVarianceStatus::Cancelled;
            }
            LenientGrantVarianceStatus::InProgress => {
                match next {
                    LenientGrantVarianceStatus::Completed | LenientGrantVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientGrantVarianceStatus::Completed => {
                return next == LenientGrantVarianceStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientGrantVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient grant variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.grace_days as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientGrantVariance {
        LenientGrantVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            checksum: self.checksum.clone(),
            grace_days: self.grace_days,
            weight: self.weight,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a lenient grant variance in status planned.
pub fn new_lenient_grant_variance(code: &str, name: &str) -> LenientGrantVariance {
    LenientGrantVariance {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        checksum: "".to_string(),
        grace_days: 0,
        weight: 0.0,
        channel: "".to_string(),
        status: LenientGrantVarianceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_grant_variance("code-559", "name-872");
        assert_eq!(item.status, LenientGrantVarianceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_lenient_grant_variance("code-559", "name-872");
        if item.move_to(LenientGrantVarianceStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientGrantVarianceStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_grant_variance("code-559", "name-872");
        item.status = LenientGrantVarianceStatus::Cancelled;
        assert!(
            item.move_to(LenientGrantVarianceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
