use log::debug;
use crate::state::State;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {
    // replace env_logger:init line with console.log for wasm
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }
    
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();


    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(&window).await;
    let mut surface_configured = false;
    let _ = event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(new_size) => {
                        surface_configured = true;
                        state.resize(*new_size);
                    },
                    WindowEvent::RedrawRequested => {
                        state.window().request_redraw();

                        if !surface_configured {
                            return;
                        }

                        state.update();
                        match state.render() {
                            Ok(_) => {}

                            // Reconfigure the surface if it's lost or outdated
                            Err(
                                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                            ) => state.resize(state.size),

                            // quite if OOM
                            Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other,) => {
                                log::error!("Out of memory!");
                                control_flow.exit();
                            }

                            // log when surface timeout (when a frame takes too long to displace)
                            Err(wgpu::SurfaceError::Timeout,) => {
                                log::warn!("Surface timeout!")
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });
}