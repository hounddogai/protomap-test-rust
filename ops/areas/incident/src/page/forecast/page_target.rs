use super::*;

use acme_kit::text;

/// Where a page target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PageTargetStatus {
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

impl std::fmt::Display for PageTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page target as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageTarget {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub amount: f64,
    pub rate: f64,
    pub maximum: f64,
    pub timezone: String,
    pub status: PageTargetStatus,
}

impl PageTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageTargetStatus) -> bool {
        match self.status {
            PageTargetStatus::Idle => {
                return next == PageTargetStatus::Ready || next == PageTargetStatus::Stopped;
            }
            PageTargetStatus::Warming => {
                match next {
                    PageTargetStatus::Ready | PageTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageTargetStatus::Ready => {
                return next == PageTargetStatus::Stopped;
            }
            PageTargetStatus::Draining => {
                match next {
                    PageTargetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PageTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        (self.amount * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageTarget {
        PageTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            amount: self.amount,
            rate: self.rate,
            maximum: self.maximum,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a page target in status idle.
pub fn new_page_target(code: &str, title: &str) -> PageTarget {
    PageTarget {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        amount: 0.0,
        rate: 0.0,
        maximum: 0.0,
        timezone: "".to_string(),
        status: PageTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_page_target("code-443", "title-510");
        assert_eq!(item.status, PageTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_page_target("code-443", "title-510");
        if item.move_to(PageTargetStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageTargetStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_page_target("code-443", "title-510");
        item.status = PageTargetStatus::Stopped;
        assert!(item.move_to(PageTargetStatus::Idle).is_err(), "expected move to fail");
    }
}
