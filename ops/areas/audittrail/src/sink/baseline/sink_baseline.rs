use super::*;

use acme_kit::text;

/// Where a sink baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkBaselineStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SinkBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for SinkBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One sink baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkBaseline {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub principal: String,
    pub retries: i64,
    pub maximum: f64,
    pub segment: String,
    pub status: SinkBaselineStatus,
}

impl SinkBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkBaselineStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkBaselineStatus) -> bool {
        match self.status {
            SinkBaselineStatus::Draft => {
                return next == SinkBaselineStatus::Active || next == SinkBaselineStatus::Closed;
            }
            SinkBaselineStatus::Pending => {
                match next {
                    SinkBaselineStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkBaselineStatus::Active => {
                match next {
                    SinkBaselineStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkBaselineStatus::Suspended => {
                return next == SinkBaselineStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "sink baseline", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.retries as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkBaseline {
        SinkBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            principal: self.principal.clone(),
            retries: self.retries,
            maximum: self.maximum,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a sink baseline in status draft.
pub fn new_sink_baseline(code: &str, name: &str) -> SinkBaseline {
    SinkBaseline {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        principal: "".to_string(),
        retries: 0,
        maximum: 0.0,
        segment: "".to_string(),
        status: SinkBaselineStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_sink_baseline("code-837", "name-843");
        assert_eq!(item.status, SinkBaselineStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_sink_baseline("code-837", "name-843");
        if item.move_to(SinkBaselineStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkBaselineStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_sink_baseline("code-837", "name-843");
        item.status = SinkBaselineStatus::Closed;
        assert!(
            item.move_to(SinkBaselineStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
