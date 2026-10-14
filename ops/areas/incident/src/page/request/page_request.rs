use super::*;

use acme_kit::text;

/// Where a page request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageRequestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PageRequestStatus {
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

impl std::fmt::Display for PageRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One page request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageRequest {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub rotation_days: i64,
    pub visible: bool,
    pub sequence: i64,
    pub region: String,
    pub retries: i64,
    pub budget: f64,
    pub window_days: i64,
    pub locale: String,
    pub status: PageRequestStatus,
}

impl PageRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageRequestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageRequestStatus) -> bool {
        match self.status {
            PageRequestStatus::Idle => {
                match next {
                    PageRequestStatus::Warming | PageRequestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageRequestStatus::Warming => {
                match next {
                    PageRequestStatus::Ready | PageRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageRequestStatus::Ready => {
                return next == PageRequestStatus::Stopped;
            }
            PageRequestStatus::Draining => {
                match next {
                    PageRequestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PageRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.retries as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageRequest {
        PageRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            rotation_days: self.rotation_days,
            visible: self.visible,
            sequence: self.sequence,
            region: self.region.clone(),
            retries: self.retries,
            budget: self.budget,
            window_days: self.window_days,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a page request in status idle.
pub fn new_page_request(code: &str, label: &str) -> PageRequest {
    PageRequest {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        rotation_days: 0,
        visible: false,
        sequence: 0,
        region: "".to_string(),
        retries: 0,
        budget: 0.0,
        window_days: 0,
        locale: "".to_string(),
        status: PageRequestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_page_request("code-347", "label-599");
        assert_eq!(item.status, PageRequestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_page_request("code-347", "label-599");
        if item.move_to(PageRequestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageRequestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_page_request("code-347", "label-599");
        item.status = PageRequestStatus::Stopped;
        assert!(item.move_to(PageRequestStatus::Idle).is_err(), "expected move to fail");
    }
}
