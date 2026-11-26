//! 应用描述表校验。

use super::Runtime;
use crate::model::{App, EVENT_NONE, MenuAction, PageRoute};

impl Runtime<'_> {
    /// 检查应用描述表，返回第一处不一致的位置。
    pub(super) fn validate_app(app: &'static App) -> Result<(), (Option<&'static PageRoute>, u16)> {
        if !app
            .routes
            .iter()
            .any(|route| core::ptr::eq(*route, app.root_page))
        {
            return Err((None, 0));
        }

        for (index, route) in app.routes.iter().enumerate() {
            match route {
                PageRoute::Home(page) => {
                    for item in page.items {
                        if item.icon.is_empty() {
                            return Err((Some(*route), index as u16));
                        }
                    }
                }
                PageRoute::Menu(page) => {
                    if let Some(states) = page.states
                        && states.len() < page.items.len().div_ceil(4)
                    {
                        return Err((Some(*route), index as u16));
                    }

                    for item in page.items {
                        let valid = match item.action {
                            MenuAction::Page(_) | MenuAction::Int(_) | MenuAction::Bool(_) => true,
                            MenuAction::Confirm(_) => true,
                            MenuAction::Event(event) => event != EVENT_NONE,
                        };
                        if !valid {
                            return Err((Some(*route), index as u16));
                        }
                    }
                }
                PageRoute::Info(_) | PageRoute::Custom(_) => {}
            }
        }
        for (index, binding) in app.int_bindings.iter().enumerate() {
            let value = binding.value.get();
            if binding.minimum > binding.maximum
                || binding.step == 0
                || value < binding.minimum
                || value > binding.maximum
            {
                return Err((None, index as u16));
            }
        }
        Ok(())
    }
}
