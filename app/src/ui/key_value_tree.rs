use derive_new::new;
use egui::{Align, Color32, Layout, RichText, Ui};

use crate::ui::colors::WHITE;

/// Wrapper around String that implements Into from various data types
pub struct Value {
    val: String,
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value {
            val: format!("{:02X}", value),
        }
    }
}
impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value {
            val: format!("{:04X}", value),
        }
    }
}
impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Value {
            val: format!("{:06X}", value),
        }
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        Value { val: value }
    }
}
impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value {
            val: value.to_string(),
        }
    }
}
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value {
            val: value.to_string(),
        }
    }
}

const EMPTY: [KvNode; 0] = [];
#[derive(new)]
pub struct KvNode<'a> {
    #[new(into)]
    name: String,
    #[new(into)]
    value: Value,
    #[new(value = "&EMPTY")]
    children: &'a [KvNode<'a>],
}

impl<'a> KvNode<'a> {
    pub fn with_children(
        name: impl Into<String>,
        value: impl Into<Value>,
        children: &'a [KvNode],
    ) -> KvNode<'a> {
        KvNode {
            name: name.into(),
            value: value.into(),
            children,
        }
    }
}

pub fn value_tree(ui: &mut Ui, value: &KvNode, indent: usize, colors: &[Color32]) {
    ui.columns(2, |cols| {
        cols[0].with_layout(Layout::left_to_right(Align::TOP), |ui| {
            ui.add_space(8.0 * indent as f32);
            ui.label(RichText::new(value.name.clone()).color(colors[0]));
        });
        cols[1].label(RichText::new(value.value.val.clone()).color(WHITE));
    });
    for child in value.children {
        value_tree(
            ui,
            &child,
            indent + 1,
            if colors.len() == 1 {
                colors
            } else {
                &colors[1..]
            },
        );
    }
}
