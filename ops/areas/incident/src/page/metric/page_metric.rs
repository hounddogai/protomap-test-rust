use super::*;

use acme_kit::text;

/// Where a page metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PageMetricStatus {
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

impl std::fmt::Display for PageMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One page metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageMetric {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub tags: Vec<String>,
    pub ends_at: i64,
    pub phone: String,
    pub status: PageMetricStatus,
}

impl PageMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageMetricStatus) -> bool {
        match self.status {
            PageMetricStatus::Idle => {
                return next == PageMetricStatus::Draining;
            }
            PageMetricStatus::Warming => {
                return next == PageMetricStatus::Ready || next == PageMetricStatus::Draining;
            }
            PageMetricStatus::Ready => {
                match next {
                    PageMetricStatus::Draining | PageMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageMetricStatus::Draining => {
                match next {
                    PageMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PageMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page metric {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> PageMetric {
        PageMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            tags: self.tags.clone(),
            ends_at: self.ends_at,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a page metric in status idle.
pub fn new_page_metric(code: &str, name: &str) -> PageMetric {
    PageMetric {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        tags: Vec::new(),
        ends_at: 0,
        phone: "".to_string(),
        status: PageMetricStatus::Idle,
    }
}
