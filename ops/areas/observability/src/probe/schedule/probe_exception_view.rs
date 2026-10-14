use super::*;

use acme_kit::text;

/// How screens and exports show a probe exception.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeExceptionView {
    pub code: String,
    pub title: String,
    pub ratio_text: String,
    pub issuer_text: String,
    pub label_text: String,
    pub status_label: String,
}

pub fn to_probe_exception_view(item: &ProbeException) -> ProbeExceptionView {
    ProbeExceptionView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        issuer_text: item.issuer.clone(),
        label_text: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_exception_views(items: &[ProbeException]) -> Vec<ProbeExceptionView> {
    let mut views: Vec<ProbeExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_exception_view(item));
    }
    views
}

pub fn sum_probe_exception_view(view: &ProbeExceptionView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_exception("code-576", "title-523");
        let view = to_probe_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
