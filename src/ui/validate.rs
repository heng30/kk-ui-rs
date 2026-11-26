//! 应用描述表校验。

use super::Runtime;
use crate::model::{App, EVENT_NONE, MenuItemType, PAGE_NONE, PageId, PageType};

impl Runtime<'_> {
    /// 检查应用描述表，返回第一处不一致的位置。
    pub(super) fn validate_app(app: &'static App) -> Result<(), (PageId, u16)> {
        if app.root_page == PAGE_NONE || app.root_page as usize > app.routes.len() {
            return Err((0, 0));
        }
        for (index, route) in app.routes.iter().enumerate() {
            let count = match route.kind {
                PageType::Home => app.home_pages.len(),
                PageType::Menu => app.menu_pages.len(),
                PageType::Info => app.info_pages.len(),
                PageType::Custom => app.custom_pages.len(),
            };
            if route.index as usize >= count {
                return Err((index as u16 + 1, route.index));
            }
        }
        for (index, page) in app.home_pages.iter().enumerate() {
            for item in page.items {
                if item.target == PAGE_NONE
                    || item.target as usize > app.routes.len()
                    || item.icon.is_empty()
                {
                    return Err((0, index as u16));
                }
            }
        }
        for (index, page) in app.menu_pages.iter().enumerate() {
            if let Some(states) = page.states
                && states.len() < page.items.len().div_ceil(4)
            {
                return Err((0, index as u16));
            }
            for item in page.items {
                let valid = match item.kind {
                    MenuItemType::Page => {
                        item.reference != PAGE_NONE && item.reference as usize <= app.routes.len()
                    }
                    MenuItemType::Action => item.reference != EVENT_NONE,
                    MenuItemType::Int => (item.reference as usize) < app.int_bindings.len(),
                    MenuItemType::Bool => (item.reference as usize) < app.bool_bindings.len(),
                    MenuItemType::Confirm => (item.reference as usize) < app.confirm_descs.len(),
                };
                if !valid {
                    return Err((0, index as u16));
                }
            }
        }
        for (index, binding) in app.int_bindings.iter().enumerate() {
            let value = binding.value.get();
            if binding.minimum > binding.maximum
                || binding.step == 0
                || value < binding.minimum
                || value > binding.maximum
            {
                return Err((0, index as u16));
            }
        }
        Ok(())
    }
}
