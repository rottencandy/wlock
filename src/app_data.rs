use wayland_client::{protocol::{wl_registry, wl_compositor, wl_subcompositor, wl_shm, wl_seat, wl_keyboard, wl_pointer, wl_output, wl_surface, wl_subsurface}, Connection, Dispatch, QueueHandle, WEnum};
use wayland_protocols::ext::session_lock::v1::client::{ext_session_lock_manager_v1, ext_session_lock_v1, ext_session_lock_surface_v1};
use xkbcommon::xkb::{Keymap, Context, State};

pub struct Surface {
    pub name: u32,
    pub output: wl_output::WlOutput,
    pub surface: Option<wl_surface::WlSurface>,
    pub child: Option<wl_surface::WlSurface>,
    pub subsurface: Option<wl_subsurface::WlSubsurface>,
    pub lock_surface: Option<ext_session_lock_surface_v1::ExtSessionLockSurfaceV1>,
}

pub struct AppData {
    pub locked: bool,
    pub running: bool,
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub seat: Option<wl_seat::WlSeat>,
    pub seat_ptr: Option<wl_pointer::WlPointer>,
    pub seat_kb: Option<wl_keyboard::WlKeyboard>,
    pub subcompositor: Option<wl_subcompositor::WlSubcompositor>,
    pub shm: Option<wl_shm::WlShm>,
    pub surfaces: Vec<Surface>,
    pub lock_mgr: Option<ext_session_lock_manager_v1::ExtSessionLockManagerV1>,

    pub xkb_context: Context,
    pub xkb_keymap: Option<Keymap>,
    pub xkb_state: Option<State>,

    pub width: u32,
    pub height: u32,
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
        ) {
        if let wl_registry::Event::Global {
            name,
            version,
            interface,
            ..
        } = event {
            //println!("[{}] {} ({})", name, interface, version);
            match &interface[..] {
                "wl_compositor" => {
                    let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(name, version, qh, ());
                    state.compositor = Some(compositor);
                }
                "wl_seat" => {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, version, qh, ());
                    state.seat = Some(seat);
                }
                "wl_subcompositor" => {
                    let subcompositor =
                        registry.bind::<wl_subcompositor::WlSubcompositor, _, _>(name, version, qh, ());
                    state.subcompositor = Some(subcompositor);
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, version, qh, ());
                    state.shm = Some(shm);
                }
                "wl_output" => {
                    let output = registry.bind::<wl_output::WlOutput, _, _>(name, version, qh, ());
                    state.surfaces.push(Surface {
                        output,
                        name,
                        surface: None,
                        child: None,
                        subsurface: None,
                        lock_surface: None,
                    });
                }
                "ext_session_lock_manager_v1" => {
                    let lock_mgr = registry.bind::<ext_session_lock_manager_v1::ExtSessionLockManagerV1, _, _>(name, version, qh, ());
                    state.lock_mgr = Some(lock_mgr);
                }
                _ => {}
            }
        } else if let wl_registry::Event::GlobalRemove {
            name,
            ..
        } = event {
            // todo switch to drain_filter
            let mut i = 0;
            while i < state.surfaces.len() {
                if state.surfaces[i].name == name {
                    state.surfaces.remove(i);
                } else {
                    i += 1;
                }
            }
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        // no event
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for AppData {
    fn event(
        state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
        ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
            ..
        } = event {
            if state.seat_kb.is_some() {
                state.seat_kb = None;
            }
            if state.seat_ptr.is_some() {
                state.seat_ptr = None;
            }
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                state.seat_kb = Some(seat.get_keyboard(qh, ()));
            }
            if capabilities.contains(wl_seat::Capability::Pointer) {
                state.seat_ptr = Some(seat.get_pointer(qh, ()));
            }
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for AppData {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        if let wl_keyboard::Event::Key { key, .. } = event {
            state.locked = false;
            println!("Key: {}", key);
            if key == 1 {
                println!("Esc key pressed!");
                // ESC key?
                // todo
            }
        }

        if let wl_keyboard::Event::Keymap { format, .. } = event {
            if let WEnum::Value(wl_keyboard::KeymapFormat::XkbV1) = format {
                //let keymap = Keymap::new_from_file(
                //    &state.xkb_context,
                //    &mut File::from(fd),
                //    XKB_KEYMAP_FORMAT_TEXT_V1,
                //    XKB_KEYMAP_COMPILE_NO_FLAGS);
                //state.xkb_state = Some(State::new(&keymap.as_ref().unwrap()));
                //state.xkb_keymap = keymap;
            } else {
                panic!("Unknown keymap format!");
            }
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_pointer::WlPointer,
        _: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        // todo
    }
}

impl Dispatch<wl_subcompositor::WlSubcompositor, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_subcompositor::WlSubcompositor,
        _: wl_subcompositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        // no event
    }
}

impl Dispatch<wl_shm::WlShm, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_shm::WlShm,
        _: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        // no event
    }
}

impl Dispatch<wl_output::WlOutput, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_output::WlOutput,
        _: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        // todo
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        _: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        // todo
    }
}

impl Dispatch<wl_subsurface::WlSubsurface, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_subsurface::WlSubsurface,
        _: wl_subsurface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        // todo
    }
}

impl Dispatch<ext_session_lock_manager_v1::ExtSessionLockManagerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &ext_session_lock_manager_v1::ExtSessionLockManagerV1,
        _: ext_session_lock_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
        ) {
        // no event
    }
}

impl Dispatch<ext_session_lock_v1::ExtSessionLockV1, ()> for AppData {
    fn event(
        state: &mut Self,
        _: &ext_session_lock_v1::ExtSessionLockV1,
        event: ext_session_lock_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
        ) {
        match event {
            ext_session_lock_v1::Event::Finished => {
                panic!("Unable to lock session!");
            }
            ext_session_lock_v1::Event::Locked => {
                state.locked = true;
                println!("Session successfully locked!");

            }
            _ => {}
        }
    }
}

impl Dispatch<ext_session_lock_surface_v1::ExtSessionLockSurfaceV1, ()> for AppData {
    fn event(
        state: &mut Self,
        lock_surf: &ext_session_lock_surface_v1::ExtSessionLockSurfaceV1,
        event: ext_session_lock_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
        ) {
        if let ext_session_lock_surface_v1::Event::Configure { serial, width, height } = event {
            state.width = width;
            state.height = height;
            lock_surf.ack_configure(serial);
        }
    }
}
