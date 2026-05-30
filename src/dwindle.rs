use crate::manager::SendHWND;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone)]
pub struct DwindleNode {
    pub box_area: Rect,
    pub split_ratio: f32,
    pub split_top: bool, // true = vertical split (top/bottom), false = horizontal (left/right)
    pub hwnd: Option<SendHWND>,
    pub children: Option<(Box<DwindleNode>, Box<DwindleNode>)>,
}

impl DwindleNode {
    pub fn new_leaf(hwnd: SendHWND, area: Rect) -> Self {
        Self {
            box_area: area,
            split_ratio: 1.0, // 1:1 split by default
            split_top: false,
            hwnd: Some(hwnd),
            children: None,
        }
    }

    pub fn split(&mut self, new_hwnd: SendHWND) {
        if self.children.is_some() {
            // TODO: logic for nested splitting (hyprland-style mouse focal point)
            return;
        }

        let old_hwnd = self.hwnd.take().unwrap();
        
        // hyprland's logic: if height * multiplier > width, split top/bottom
        // for now let's just toggle based on aspect ratio
        self.split_top = self.box_area.height > self.box_area.width;

        let (box1, box2) = if self.split_top {
            let h1 = (self.box_area.height as f32 / 2.0 * self.split_ratio) as i32;
            (
                Rect { x: self.box_area.x, y: self.box_area.y, width: self.box_area.width, height: h1 },
                Rect { x: self.box_area.x, y: self.box_area.y + h1, width: self.box_area.width, height: self.box_area.height - h1 }
            )
        } else {
            let w1 = (self.box_area.width as f32 / 2.0 * self.split_ratio) as i32;
            (
                Rect { x: self.box_area.x, y: self.box_area.y, width: w1, height: self.box_area.height },
                Rect { x: self.box_area.x + w1, y: self.box_area.y, width: self.box_area.width - w1, height: self.box_area.height }
            )
        };

        self.children = Some((
            Box::new(DwindleNode::new_leaf(old_hwnd, box1)),
            Box::new(DwindleNode::new_leaf(new_hwnd, box2))
        ));
    }

    // walk the tree and get all windows + their calculated rects
    pub fn get_layout_results(&self, results: &mut Vec<(SendHWND, Rect)>) {
        if let Some(hwnd) = self.hwnd {
            results.push((hwnd, self.box_area));
        }
        if let Some((child1, child2)) = &self.children {
            child1.get_layout_results(results);
            child2.get_layout_results(results);
        }
    }
}
