use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::num::NonZeroU32;
use winit::window::Window;
/*
    Draw a filled rectangle on the buffer.
    @param buffer: The pixel buffer to draw on.
    @param buffer_width: The width of the buffer in pixels.
    @param x: The x-coordinate of the top-left corner of the rectangle.
    @param y: The y-coordinate of the top-left corner of the rectangle.
    @param width: The width of the rectangle in pixels.
    @param height: The height of the rectangle in pixels.
    @param color: The color of the rectangle in ARGB format (0xAARRGGBB).
 */
fn draw_rect(
    buffer: &mut [u32],
    buffer_width: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: u32
) {
    for current_y in y..(y + height) {
        for current_x in x..(x + width) {
            if current_x >= buffer_width { continue; }
            let index = (current_y * buffer_width) + current_x;
            if index < buffer.len() {
                buffer[index] = color;
            }
        }
    }
}

/*
    Initializes the FAGA Browser application, creating a window and setting up the rendering context.
    This function will block and run the event loop until the window is closed.
*/
pub fn init() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("FAGA Browser")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    let context = { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = { softbuffer::Surface::new(&context, &window) }.unwrap();

    log::info!("🎨 FAGA Core: Prêt à dessiner.");

    event_loop.set_control_flow(ControlFlow::Wait);

    run(event_loop, &window, surface);
}

/*
    Runs the main event loop for the FAGA Browser application, handling window events and rendering.
    @param event_loop: The event loop to run.
    @param window: The window to render on.
    @param surface: The softbuffer surface for drawing.
 */
fn run(
    event_loop: EventLoop<()>,
    window: &Window,
    mut surface: softbuffer::Surface<&Window, &Window>,
) {
    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                elwt.exit();
            },

            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == window.id() => {
                // 1. Récupérer les dimensions
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };

                // 2. Redimensionner la mémoire vidéo si besoin
                if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
                    surface.resize(w, h).unwrap();
                    let mut buffer = surface.buffer_mut().unwrap();
                    let buffer_width = width as usize;
                    let buffer_height = height as usize;


                    buffer.fill(0x00101030);

                    draw_rect(&mut buffer, buffer_width,
                              0, 0, buffer_width, 50,
                              0x00404040
                    );

                    if buffer_width > 40 && buffer_height > 70 {
                        draw_rect(&mut buffer, buffer_width,
                                  20, 70,
                                  buffer_width - 40,
                                  buffer_height - 90,
                                  0x00FFFFFF
                        );
                    }

                    buffer.present().unwrap();
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(..),
                ..
            } => {
                window.request_redraw();
            }

            _ => ()
        }

    });
}