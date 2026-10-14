use super::*;

use acme_kit::text;

/// Where a page threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageThresholdStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PageThresholdStatus {
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

impl std::fmt::Display for PageThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PageThreshold {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub attempts: i64,
    pub window_days: i64,
    pub status: PageThresholdStatus,
}

impl PageThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageThresholdStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageThresholdStatus) -> bool {
        match self.status {
            PageThresholdStatus::Idle => {
                return next == PageThresholdStatus::Ready;
            }
            PageThresholdStatus::Warming => {
                return next == PageThresholdStatus::Ready;
            }
            PageThresholdStatus::Ready => {
                return next == PageThresholdStatus::Draining || next == PageThresholdStatus::Stopped;
            }
            PageThresholdStatus::Draining => {
                match next {
                    PageThresholdStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PageThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page threshold {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.window_days as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageThreshold {
        PageThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            attempts: self.attempts,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a page threshold in status idle.
pub fn new_page_threshold(code: &str, label: &str) -> PageThreshold {
    PageThreshold {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        attempts: 0,
        window_days: 0,
        status: PageThresholdStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_page_threshold("code-128", "label-890");
        assert_eq!(item.status, PageThresholdStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_page_threshold("code-128", "label-890");
        if item.move_to(PageThresholdStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageThresholdStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_page_threshold("code-128", "label-890");
        item.status = PageThresholdStatus::Stopped;
        assert!(
            item.move_to(PageThresholdStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
