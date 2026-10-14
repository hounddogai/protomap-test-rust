use super::*;

use acme_kit::text;

/// Where a manual redaction metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRedactionMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualRedactionMetricStatus {
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

impl std::fmt::Display for ManualRedactionMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual redaction metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionMetric {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub fingerprint: String,
    pub amount: f64,
    pub priority: i64,
    pub visible: bool,
    pub namespace: String,
    pub status: ManualRedactionMetricStatus,
}

impl ManualRedactionMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRedactionMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRedactionMetricStatus) -> bool {
        match self.status {
            ManualRedactionMetricStatus::Idle => {
                return next == ManualRedactionMetricStatus::Ready;
            }
            ManualRedactionMetricStatus::Warming => {
                return next == ManualRedactionMetricStatus::Draining || next == ManualRedactionMetricStatus::Stopped;
            }
            ManualRedactionMetricStatus::Ready => {
                match next {
                    ManualRedactionMetricStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRedactionMetricStatus::Draining => {
                return next == ManualRedactionMetricStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualRedactionMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual redaction metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.amount - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRedactionMetric {
        ManualRedactionMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            fingerprint: self.fingerprint.clone(),
            amount: self.amount,
            priority: self.priority,
            visible: self.visible,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a manual redaction metric in status idle.
pub fn new_manual_redaction_metric(code: &str, title: &str) -> ManualRedactionMetric {
    ManualRedactionMetric {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        fingerprint: "".to_string(),
        amount: 0.0,
        priority: 0,
        visible: false,
        namespace: "".to_string(),
        status: ManualRedactionMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_redaction_metric("code-822", "title-22");
        assert_eq!(item.status, ManualRedactionMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_manual_redaction_metric("code-822", "title-22");
        if item.move_to(ManualRedactionMetricStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRedactionMetricStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_redaction_metric("code-822", "title-22");
        item.status = ManualRedactionMetricStatus::Stopped;
        assert!(
            item.move_to(ManualRedactionMetricStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
