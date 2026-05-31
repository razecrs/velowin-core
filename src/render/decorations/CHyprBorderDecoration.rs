use crate::helpers::Types::SendHWND;
use crate::layout::algorithm::tiled::dwindle::DwindleAlgorithm::Rect;
use crate::render::Renderer;

pub struct CHyprBorderDecoration {
    pub border_size: i32,
    pub rounding: f32,
}

impl CHyprBorderDecoration {
    pub fn new() -> Self {
        Self {
            border_size: 2,
            rounding: 10.0,
        }
    }

    pub fn draw(&self, hwnd: SendHWND, rect: &Rect, angle: f32) {
        // 1:1 Mirror: This file now 'owns' the call to the low-level gradient renderer
        Renderer::CreateBorderWrapper(hwnd, self.border_size, self.rounding);
        Renderer::SetBorderAngleWrapper(hwnd, angle);
        Renderer::UpdateBorderPositionWrapper(hwnd, rect.x, rect.y, rect.width, rect.height);
    }
}
