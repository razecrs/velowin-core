use crate::layout::algorithm::tiled::dwindle::DwindleAlgorithm::DwindleNode;

pub struct Workspace {
    pub id: u32,
    pub root: Option<DwindleNode>,
    pub layout: LayoutType,
}

pub enum LayoutType {
    Dwindle,
    Master,
}
