use super::*;

use acme_kit::text;

/// Where a regional redaction group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRedactionGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalRedactionGroupStatus {
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

impl std::fmt::Display for RegionalRedactionGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional redaction group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionGroup {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub audience: String,
    pub minimum: f64,
    pub external_ref: String,
    pub category: String,
    pub ttl_seconds: i64,
    pub port: i64,
    pub status: RegionalRedactionGroupStatus,
}

impl RegionalRedactionGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRedactionGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRedactionGroupStatus) -> bool {
        match self.status {
            RegionalRedactionGroupStatus::Idle => {
                return next == RegionalRedactionGroupStatus::Ready || next == RegionalRedactionGroupStatus::Stopped;
            }
            RegionalRedactionGroupStatus::Warming => {
                match next {
                    RegionalRedactionGroupStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRedactionGroupStatus::Ready => {
                return next == RegionalRedactionGroupStatus::Stopped;
            }
            RegionalRedactionGroupStatus::Draining => {
                return next == RegionalRedactionGroupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalRedactionGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional redaction group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.minimum - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRedactionGroup {
        RegionalRedactionGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            audience: self.audience.clone(),
            minimum: self.minimum,
            external_ref: self.external_ref.clone(),
            category: self.category.clone(),
            ttl_seconds: self.ttl_seconds,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a regional redaction group in status idle.
pub fn new_regional_redaction_group(code: &str, label: &str) -> RegionalRedactionGroup {
    RegionalRedactionGroup {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        audience: "".to_string(),
        minimum: 0.0,
        external_ref: "".to_string(),
        category: "".to_string(),
        ttl_seconds: 0,
        port: 0,
        status: RegionalRedactionGroupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_redaction_group("code-379", "label-322");
        assert_eq!(item.status, RegionalRedactionGroupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_regional_redaction_group("code-379", "label-322");
        if item.move_to(RegionalRedactionGroupStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRedactionGroupStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_redaction_group("code-379", "label-322");
        item.status = RegionalRedactionGroupStatus::Stopped;
        assert!(
            item.move_to(RegionalRedactionGroupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
