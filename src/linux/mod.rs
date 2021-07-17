pub mod keyboard_mouse;

use crate::details::lock_registry;
use crate::keyboard_mouse::{kb_code_to_key, mouse_code_to_key};
use crate::Event;
use input::event::keyboard::{KeyState, KeyboardEventTrait};
use input::event::pointer::ButtonState;
use input::event::pointer::PointerEvent::Button;
use input::{Libinput, LibinputInterface};
use nix::fcntl::{open, OFlag};
use nix::poll::{poll, PollFd, PollFlags};
use nix::sys::stat::Mode;
use nix::unistd::close;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::path::Path;

pub(crate) fn install_hooks() {}

pub(crate) fn process_message() {
    struct LibinputInterfaceRaw;

    impl LibinputInterface for LibinputInterfaceRaw {
        fn open_restricted(&mut self, path: &Path, flags: i32) -> std::result::Result<RawFd, i32> {
            if let Ok(fd) = open(path, OFlag::from_bits_truncate(flags), Mode::empty()) {
                Ok(fd)
            } else {
                Err(1)
            }
        }

        fn close_restricted(&mut self, fd: RawFd) {
            let _ = close(fd);
        }
    }
    let mut libinput = Libinput::new_with_udev(LibinputInterfaceRaw);
    libinput.udev_assign_seat(&"seat0").unwrap();
    let pollfd = PollFd::new(libinput.as_raw_fd(), PollFlags::POLLIN);
    while let Ok(_) = poll(&mut [pollfd], -1) {
        libinput.dispatch().unwrap();
        while let Some(event) = libinput.next() {
            handle_libinput_event(event);
        }
    }
}

fn handle_libinput_event(event: input::Event) {
    match event {
        input::Event::Device(_) => {}
        input::Event::Keyboard(kb) => {
            let key = kb_code_to_key(kb.key());
            match kb.key_state() {
                KeyState::Pressed => {
                    lock_registry().event_down(Event::Keyboard(key));
                }
                KeyState::Released => {
                    lock_registry().event_up(Event::Keyboard(key));
                }
            }
        }
        input::Event::Pointer(pointer) => {
            if let Button(button_event) = pointer {
                if let Some(mapped) = mouse_code_to_key(button_event.button()) {
                    match button_event.button_state() {
                        ButtonState::Pressed => {
                            lock_registry().event_down(Event::Mouse(mapped));
                        }
                        ButtonState::Released => {
                            lock_registry().event_up(Event::Mouse(mapped));
                        }
                    }
                }
            }
        }
        input::Event::Touch(_) => {}
        input::Event::Tablet(_) => {}
        input::Event::TabletPad(_) => {}
        input::Event::Gesture(_) => {}
        input::Event::Switch(_) => {}
    }
}
