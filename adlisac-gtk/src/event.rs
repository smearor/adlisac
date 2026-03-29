// use adlisac_core::state::base::AdlisacState;
// use glib::ControlFlow;
// use smithay::reexports::wayland_server::Display;
// use std::cell::RefCell;
// use std::rc::Rc;
//
// pub struct EventLoopBridge {
//     display: Rc<RefCell<Display<AdlisacState>>>,
//     state: Rc<RefCell<AdlisacState>>,
// }
//
// impl EventLoopBridge {
//     pub fn attach_to_glib(self) {
//         // Wir holen uns den File-Descriptor des Wayland-Backends
//         let fd = self.display.borrow().backend().poll_fd();
//
//         // Wir fügen diesen FD zur Standard-GLib-Loop hinzu
//         let main_context = glib::MainContext::default();
//         main_context.add_unix_fd(fd, glib::IOCondition::IN | glib::IOCondition::ERR | glib::IOCondition::HUP, move |_, _| {
//             // Diese Logik wird jedes Mal aufgerufen, wenn
//             // ein Wayland-Client (z.B. Firefox) Daten sendet.
//
//             // 1. Events vom Socket lesen und dispatchen
//             // 2. Display flushen (Antworten an Clients senden)
//
//             ControlFlow::Continue
//         });
//     }
// }
