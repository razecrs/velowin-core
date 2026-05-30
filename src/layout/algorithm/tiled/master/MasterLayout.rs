use crate::helpers::Types::SendHWND;
use crate::layout::algorithm::tiled::dwindle::DwindleAlgorithm::Rect;

pub struct MasterNode {
    pub hwnd: SendHWND,
    pub box_area: Rect,
}

pub struct MasterLayout {
    pub windows: Vec<MasterNode>,
    pub master_count: i32,
    pub master_split_ratio: f32,
}

impl MasterLayout {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            master_count: 1,
            master_split_ratio: 0.5,
        }
    }

    pub fn add_window(&mut self, hwnd: SendHWND) {
        self.windows.push(MasterNode { hwnd, box_area: Rect { x: 0, y: 0, width: 0, height: 0 } });
        self.recalculate(Rect { x: 0, y: 0, width: 1920, height: 1080 }); // TODO: get real monitor res
    }

    pub fn recalculate(&mut self, total_area: Rect) {
        if self.windows.is_empty() { return; }

        let count = self.windows.len() as i32;
        
        if count <= self.master_count {
            // All windows are masters (vertical stack)
            let h = total_area.height / count;
            for (i, win) in self.windows.iter_mut().enumerate() {
                win.box_area = Rect {
                    x: total_area.x,
                    y: total_area.y + (i as i32 * h),
                    width: total_area.width,
                    height: h,
                };
            }
        } else {
            // Master area and stack area
            let master_width = (total_area.width as f32 * self.master_split_ratio) as i32;
            let stack_width = total_area.width - master_width;
            
            let master_h = total_area.height / self.master_count;
            let stack_count = count - self.master_count;
            let stack_h = total_area.height / stack_count;

            for (i, win) in self.windows.iter_mut().enumerate() {
                let i = i as i32;
                if i < self.master_count {
                    win.box_area = Rect {
                        x: total_area.x,
                        y: total_area.y + (i * master_h),
                        width: master_width,
                        height: master_h,
                    };
                } else {
                    win.box_area = Rect {
                        x: total_area.x + master_width,
                        y: total_area.y + ((i - self.master_count) * stack_h),
                        width: stack_width,
                        height: stack_h,
                    };
                }
            }
        }
    }
}
