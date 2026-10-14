use super::*;

use acme_kit::text;

/// Where a fallback data export rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackDataExportRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackDataExportRuleStatus {
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

impl std::fmt::Display for FallbackDataExportRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback data export rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportRule {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub score: f64,
    pub maximum: f64,
    pub description: String,
    pub attempts: i64,
    pub channel: String,
    pub locale: String,
    pub ceiling: f64,
    pub retries: i64,
    pub status: FallbackDataExportRuleStatus,
}

impl FallbackDataExportRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackDataExportRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackDataExportRuleStatus) -> bool {
        match self.status {
            FallbackDataExportRuleStatus::Idle => {
                match next {
                    FallbackDataExportRuleStatus::Warming | FallbackDataExportRuleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDataExportRuleStatus::Warming => {
                match next {
                    FallbackDataExportRuleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDataExportRuleStatus::Ready => {
                match next {
                    FallbackDataExportRuleStatus::Draining | FallbackDataExportRuleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDataExportRuleStatus::Draining => {
                match next {
                    FallbackDataExportRuleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FallbackDataExportRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback data export rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback data export rule", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.maximum * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackDataExportRule {
        FallbackDataExportRule {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            score: self.score,
            maximum: self.maximum,
            description: self.description.clone(),
            attempts: self.attempts,
            channel: self.channel.clone(),
            locale: self.locale.clone(),
            ceiling: self.ceiling,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a fallback data export rule in status idle.
pub fn new_fallback_data_export_rule(code: &str, title: &str) -> FallbackDataExportRule {
    FallbackDataExportRule {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        score: 0.0,
        maximum: 0.0,
        description: "".to_string(),
        attempts: 0,
        channel: "".to_string(),
        locale: "".to_string(),
        ceiling: 0.0,
        retries: 0,
        status: FallbackDataExportRuleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_data_export_rule("code-52", "title-389");
        assert_eq!(item.status, FallbackDataExportRuleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_fallback_data_export_rule("code-52", "title-389");
        if item.move_to(FallbackDataExportRuleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackDataExportRuleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_data_export_rule("code-52", "title-389");
        item.status = FallbackDataExportRuleStatus::Stopped;
        assert!(
            item.move_to(FallbackDataExportRuleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
