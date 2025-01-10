use windows::Win32::Foundation::POINT;

use crate::UIElementProps;


// TODO: Change the return value to contain both the element and the index
//       and add the index as an input parameter as well to start looping from that index
//       as the rectangles are sorted by size
pub fn get_point_bounding_rect<'a>(point: &'a POINT, ui_elements: &'a Vec<UIElementProps>) -> Option<&'a UIElementProps> {
// pub fn get_point_bounding_rect(point: &Pos2, ui_elements: &Vec<UIElementProps>) -> Option<&UIElementProps> {
    // let mut cntr = 0;
    for element in ui_elements {
        // cntr += 1;
        if is_inside_rectancle(&element.bounding_rect, point.x, point.y) {
            // println!("point: {{ x: {}, y: {} }} searched elements: {} / Found element: {{ name: '{}', control_type: '{}' bounding_rect: {} }}", point.x, point.y, cntr, element.name, element.control_type, element.bounding_rect);        
            return Some(element);
        }
    }
    // printfmt!("NO ELEMENT FOUND! Searched elements: {}", cntr);
    None
}


pub fn is_inside_rectancle(rect: &uiautomation::types::Rect, x: i32, y: i32) -> bool {
    x >= rect.get_left() && x <= rect.get_right() && y >= rect.get_top() && y <= rect.get_bottom()
}
