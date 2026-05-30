use crate::layout::DwindleLayout::DwindleNode;

pub struct Workspace {
    pub id: u32,
    pub root: Option<DwindleNode>,
    pub layout: LayoutType,
}

pub enum LayoutType {
    Dwindle,
    Master,
}
