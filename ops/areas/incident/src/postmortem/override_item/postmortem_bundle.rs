use super::*;

use acme_kit::text;

/// Where a postmortem bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PostmortemBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PostmortemBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem bundle as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBundle {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub window_days: i64,
    pub visible: bool,
    pub channel: String,
    pub status: PostmortemBundleStatus,
}

impl PostmortemBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemBundleStatus) -> bool {
        match self.status {
            PostmortemBundleStatus::New => {
                match next {
                    PostmortemBundleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemBundleStatus::Queued => {
                match next {
                    PostmortemBundleStatus::Running | PostmortemBundleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemBundleStatus::Running => {
                return next == PostmortemBundleStatus::Done || next == PostmortemBundleStatus::Failed;
            }
            PostmortemBundleStatus::Done => {
                match next {
                    PostmortemBundleStatus::Failed => {
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

    pub fn move_to(&mut self, next: PostmortemBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.rate + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemBundle {
        PostmortemBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            window_days: self.window_days,
            visible: self.visible,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a postmortem bundle in status new.
pub fn new_postmortem_bundle(code: &str, label: &str) -> PostmortemBundle {
    PostmortemBundle {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        window_days: 0,
        visible: false,
        channel: "".to_string(),
        status: PostmortemBundleStatus::New,
    }
}
