pub fn is_inside_rectancle(rect: &uiautomation::types::Rect, x: i32, y: i32) -> bool {
    x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom
}
