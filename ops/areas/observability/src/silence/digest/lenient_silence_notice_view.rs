use super::*;

use acme_kit::text;

/// How screens and exports show a lenient silence notice.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceNoticeView {
    pub code: String,
    pub title: String,
    pub archived: String,
    pub visible: String,
    pub external_ref: String,
    pub amount: String,
    pub phone: String,
    pub revision: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_lenient_silence_notice_view(item: &LenientSilenceNotice) -> LenientSilenceNoticeView {
    LenientSilenceNoticeView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        archived: format!("{}", item.archived),
        visible: format!("{}", item.visible),
        external_ref: item.external_ref.clone(),
        amount: format!("{:.2}", item.amount),
        phone: item.phone.clone(),
        revision: item.revision.to_string(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_silence_notice_views(items: &[LenientSilenceNotice]) -> Vec<LenientSilenceNoticeView> {
    let mut views: Vec<LenientSilenceNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_silence_notice_view(item));
    }
    views
}

pub fn sum_lenient_silence_notice_view(view: &LenientSilenceNoticeView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_amount + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_silence_notice("code-772", "title-919");
        let view = to_lenient_silence_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_silence_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
