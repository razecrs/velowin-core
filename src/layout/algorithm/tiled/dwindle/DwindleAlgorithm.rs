use crate::helpers::Types::SendHWND;

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
    pub split_top: bool, 
    pub hwnd: Option<SendHWND>,
    pub children: Option<(Box<DwindleNode>, Box<DwindleNode>)>,
}

impl DwindleNode {
    pub fn new_leaf(hwnd: SendHWND, area: Rect) -> Self {
        Self {
            box_area: area,
            split_ratio: 0.5, 
            split_top: false,
            hwnd: Some(hwnd),
            children: None,
        }
    }

    fn calculate_split_boxes_static(box_area: Rect, split_top: bool, split_ratio: f32) -> (Rect, Rect) {
        if split_top {
            let h1 = (box_area.height as f32 * split_ratio) as i32;
            (
                Rect { x: box_area.x, y: box_area.y, width: box_area.width, height: h1 },
                Rect { x: box_area.x, y: box_area.y + h1, width: box_area.width, height: box_area.height - h1 }
            )
        } else {
            let w1 = (box_area.width as f32 * split_ratio) as i32;
            (
                Rect { x: box_area.x, y: box_area.y, width: w1, height: box_area.height },
                Rect { x: box_area.x + w1, y: box_area.y, width: box_area.width - w1, height: box_area.height }
            )
        }
    }

    pub fn split(&mut self, new_hwnd: SendHWND) {
        if let Some((child1, child2)) = &mut self.children {
            let area1 = child1.box_area.width * child1.box_area.height;
            let area2 = child2.box_area.width * child2.box_area.height;
            
            if area1 > area2 {
                child1.split(new_hwnd);
            } else {
                child2.split(new_hwnd);
            }
            self.update_child_areas();
            return;
        }

        let old_hwnd = self.hwnd.take().expect("Leaf node must have an HWND");
        self.split_top = self.box_area.height > self.box_area.width;

        let (box1, box2) = Self::calculate_split_boxes_static(self.box_area, self.split_top, self.split_ratio);

        self.children = Some((
            Box::new(DwindleNode::new_leaf(old_hwnd, box1)),
            Box::new(DwindleNode::new_leaf(new_hwnd, box2))
        ));
    }

    pub fn remove(&mut self, target_hwnd: SendHWND) -> bool {
        if let Some(hwnd) = self.hwnd {
            return hwnd == target_hwnd;
        }

        let mut should_collapse_child1 = false;
        let mut should_collapse_child2 = false;

        if let Some((child1, child2)) = &mut self.children {
            if child1.remove(target_hwnd) {
                should_collapse_child1 = true;
            } else if child2.remove(target_hwnd) {
                should_collapse_child2 = true;
            }
        }

        if should_collapse_child1 {
            let (_, mut child2) = self.children.take().unwrap();
            if let Some(h) = child2.hwnd {
                self.hwnd = Some(h);
            } else {
                self.children = child2.children.take();
            }
            self.update_child_areas();
            return false;
        }

        if should_collapse_child2 {
            let (mut child1, _) = self.children.take().unwrap();
            if let Some(h) = child1.hwnd {
                self.hwnd = Some(h);
            } else {
                self.children = child1.children.take();
            }
            self.update_child_areas();
            return false;
        }

        false
    }

    fn update_child_areas(&mut self) {
        if let Some((child1, child2)) = &mut self.children {
            let (box1, box2) = Self::calculate_split_boxes_static(self.box_area, self.split_top, self.split_ratio);
            child1.box_area = box1;
            child2.box_area = box2;
            child1.update_child_areas();
            child2.update_child_areas();
        }
    }

    pub fn get_layout_results(&self, results: &mut Vec<(SendHWND, Rect)>, gaps_in: i32, gaps_out: i32) {
        if let Some(hwnd) = self.hwnd {
            let effective_rect = Rect {
                x: self.box_area.x + gaps_out,
                y: self.box_area.y + gaps_out,
                width: self.box_area.width - (gaps_out * 2),
                height: self.box_area.height - (gaps_out * 2),
            };
            results.push((hwnd, effective_rect));
        }
        if let Some((child1, child2)) = &self.children {
            child1.get_layout_results(results, gaps_in, gaps_out);
            child2.get_layout_results(results, gaps_in, gaps_out);
        }
    }
}
