use super::*;

use acme_kit::text;

/// How screens and exports show a global release allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAllocationView {
    pub code: String,
    pub label_text: String,
    pub burst: String,
    pub fingerprint: String,
    pub batch_size_text: String,
    pub ceiling_text: String,
    pub weight: String,
    pub checksum: String,
    pub channel_text: String,
    pub status_label: String,
}

pub fn to_global_release_allocation_view(item: &GlobalReleaseAllocation) -> GlobalReleaseAllocationView {
    GlobalReleaseAllocationView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        burst: item.burst.to_string(),
        fingerprint: item.fingerprint.clone(),
        batch_size_text: item.batch_size.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        weight: format!("{:.2}", item.weight),
        checksum: text::truncate(&item.checksum, 64),
        channel_text: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_release_allocation_views(items: &[GlobalReleaseAllocation]) -> Vec<GlobalReleaseAllocationView> {
    let mut views: Vec<GlobalReleaseAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_release_allocation_view(item));
    }
    views
}

pub fn sum_global_release_allocation_view(view: &GlobalReleaseAllocationView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_batch_size as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_release_allocation("code-926", "label-742");
        let view = to_global_release_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_release_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
