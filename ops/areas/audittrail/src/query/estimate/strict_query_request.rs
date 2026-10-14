use super::*;

use acme_kit::text;

/// Where a strict query request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictQueryRequestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictQueryRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for StrictQueryRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict query request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryRequest {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub audience: String,
    pub channel: String,
    pub locale: String,
    pub tags: Vec<String>,
    pub grace_days: i64,
    pub summary: String,
    pub maximum: f64,
    pub quota_units: i64,
    pub status: StrictQueryRequestStatus,
}

impl StrictQueryRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictQueryRequestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictQueryRequestStatus) -> bool {
        match self.status {
            StrictQueryRequestStatus::Idle => {
                return next == StrictQueryRequestStatus::Warming || next == StrictQueryRequestStatus::Ready;
            }
            StrictQueryRequestStatus::Warming => {
                match next {
                    StrictQueryRequestStatus::Ready | StrictQueryRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictQueryRequestStatus::Ready => {
                return next == StrictQueryRequestStatus::Stopped;
            }
            StrictQueryRequestStatus::Draining => {
                match next {
                    StrictQueryRequestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: StrictQueryRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict query request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.maximum - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictQueryRequest {
        StrictQueryRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            audience: self.audience.clone(),
            channel: self.channel.clone(),
            locale: self.locale.clone(),
            tags: self.tags.clone(),
            grace_days: self.grace_days,
            summary: self.summary.clone(),
            maximum: self.maximum,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a strict query request in status idle.
pub fn new_strict_query_request(code: &str, label: &str) -> StrictQueryRequest {
    StrictQueryRequest {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        audience: "".to_string(),
        channel: "".to_string(),
        locale: "".to_string(),
        tags: Vec::new(),
        grace_days: 0,
        summary: "".to_string(),
        maximum: 0.0,
        quota_units: 0,
        status: StrictQueryRequestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_query_request("code-667", "label-98");
        assert_eq!(item.status, StrictQueryRequestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_strict_query_request("code-667", "label-98");
        if item.move_to(StrictQueryRequestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictQueryRequestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_query_request("code-667", "label-98");
        item.status = StrictQueryRequestStatus::Stopped;
        assert!(
            item.move_to(StrictQueryRequestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
