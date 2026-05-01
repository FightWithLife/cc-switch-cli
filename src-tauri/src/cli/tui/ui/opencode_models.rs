#[path = "opencode_model_detail.rs"]
mod opencode_model_detail;
#[path = "opencode_model_list.rs"]
mod opencode_model_list;

pub(super) use opencode_model_detail::render_opencode_model_detail;
pub(super) use opencode_model_list::render_opencode_model_list;
