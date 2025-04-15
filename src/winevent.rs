#![allow(unused)]

use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};

use uiautomation::types::Handle;
use uiautomation::{UIAutomation, UIElement};

use win_event_hook::handles::builtins::WindowHandle;
use win_event_hook::WinEventHook;
use win_event_hook::{events::{Event, NamedEvent}, handles::OpaqueHandle};

use windows::Win32::Foundation::HWND;

#[derive(Debug)]
struct WinEventInfo {
    event: Event,
    hwnd: OpaqueHandle<WindowHandle>,
}

fn create_event_handler(tx: Sender<WinEventInfo>) -> impl Fn(Event, OpaqueHandle<WindowHandle>, i32, i32, u32, u32) {
    move |ev, ohwnd: OpaqueHandle<WindowHandle>, _, _, _, _| {
        tx.send(WinEventInfo { 
            event: ev, 
            hwnd: ohwnd 
        }).unwrap_or_else(|e| eprintln!("Failed to send event: {}", e));
    }
}

fn create_hook() -> (WinEventHook, Receiver<WinEventInfo>) {
    // Create channel for communication
    let (tx, rx): (Sender<WinEventInfo>, Receiver<WinEventInfo>) = channel();

    // Create hook config
    let config = win_event_hook::Config::builder()
        .skip_own_process()
        .with_dedicated_thread()
        .with_events(vec![
            // A hidden object is shown. The system sends this event for the following user interface elements: caret, cursor, and window object. Server applications send this event for their accessible objects.
            // Clients assume that when this event is sent by a parent object, all child objects are already displayed. Therefore, server applications do not send this event for the child objects.
            // Hidden objects include the STATE_SYSTEM_INVISIBLE flag; shown objects do not include this flag. The EVENT_OBJECT_SHOW event also indicates that the STATE_SYSTEM_INVISIBLE flag is cleared. Therefore, servers do not send the EVENT_STATE_CHANGE event in this case.
            Event::Named(NamedEvent::ObjectShow),
            // An object is hidden. The system sends this event for the following user interface elements: caret and cursor. Server applications send this event for their accessible objects.
            // When this event is generated for a parent object, all child objects are already hidden. Server applications do not send this event for the child objects.
            // Hidden objects include the STATE_SYSTEM_INVISIBLE flag; shown objects do not include this flag. The EVENT_OBJECT_HIDE event also indicates that the STATE_SYSTEM_INVISIBLE flag is set. Therefore, servers do not send the EVENT_STATE_CHANGE event in this case.            
            Event::Named(NamedEvent::ObjectHide),
            // An object has been created. The system sends this event for the following user interface elements: caret, header control, list-view control, tab control, toolbar control, tree view control, and window object. Server applications send this event for their accessible objects.
            // Before sending the event for the parent object, servers must send it for all of an object's child objects. Servers must ensure that all child objects are fully created and ready to accept IAccessible calls from clients before the parent object sends this event.
            // Because a parent object is created after its child objects, clients must make sure that an object's parent has been created before calling IAccessible::get_accParent, particularly if in-context hook functions are used.
            Event::Named(NamedEvent::ObjectCreate),
            // An object has been destroyed. The system sends this event for the following user interface elements: caret, header control, list-view control, tab control, toolbar control, tree view control, and window object. Server applications send this event for their accessible objects.
            // Clients assume that all of an object's children are destroyed when the parent object sends this event.
            // After receiving this event, clients do not call an object's IAccessible properties or methods. However, the interface pointer must remain valid as long as there is a reference count on it (due to COM rules), but the UI element may no longer be present. Further calls on the interface pointer may return failure errors; to prevent this, servers create proxy objects and monitor their life spans.            
            Event::Named(NamedEvent::ObjectDestroy),
            // An object has changed location, shape, or size. The system sends this event for the following user interface elements: caret and window objects. Server applications send this event for their accessible objects.
            // This event is generated in response to a change in the top-level object within the object hierarchy; it is not generated for any children that the object might have. For example, if the user resizes a window, the system sends this notification for the window, but not for the menu bar, title bar, scroll bar, or other objects that have also changed.
            // The system does not send this event for every non-floating child window when the parent moves. However, if an application explicitly resizes child windows as a result of resizing the parent window, the system sends multiple events for the resized children.
            // If an object's State property is set to STATE_SYSTEM_FLOATING, the server sends EVENT_OBJECT_LOCATIONCHANGE whenever the object changes location. If an object does not have this state, servers only trigger this event when the object moves in relation to its parent. For this event notification, the idChild parameter of the WinEventProc callback function identifies the child object that has changed.
            Event::Named(NamedEvent::ObjectLocationChange),
        ])
        .finish();

    // Create handler and install hook
    println!("Installing hook");
    let handler = create_event_handler(tx);
    let hook = win_event_hook::WinEventHook::install(config, handler).unwrap();
    (hook, rx)
}