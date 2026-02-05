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
    let surface = { softbuffer::Surface::new(&context, &window) }.unwrap();

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
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };

                if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
                    surface.resize(w, h).unwrap();
                    let mut buffer = surface.buffer_mut().unwrap();

                    let html = "
                        <html>
                            <body>
                                <div class=\"header\">FAGA BROWSER</div>
                                <div class=\"content\">
                                    <div class=\"card\">Carte 1</div>
                                    <div class=\"card\">Carte 2</div>
                                </div>
                            </body>
                        </html>
                    ".to_string();

                    let css = "
                        body { background: black; }
                        .header { height: 80px; background: grey; margin-bottom: 20px; }
                        .content { background: white; width: 600px; height: 400px; margin-left: 50px; }
                        .card { background: red; width: 100px; height: 100px; margin-top: 20px; margin-left: 20px; }
                    ".to_string();

                    let dom_root = html::parse(html);
                    let stylesheet = css::parse(css);


                    let style_root = css::style_tree(&dom_root, &stylesheet);

                    let mut viewport = layout::Dimensions::default();
                    viewport.content.width = width as f32;
                    viewport.content.height = height as f32;
                    let layout_root = layout::layout_tree(&style_root, viewport);


                    let display_list = paint::build_display_list(&layout_root);


                    buffer.fill(0);

                    for command in display_list {
                        match command {
                            paint::DisplayCommand::SolidColor(color, rect) => {
                                draw_rect_safe(&mut buffer, width as usize, rect, color);
                            }
                        }
                    }

                    buffer.present().unwrap();
                }
            }
            Event::WindowEvent { event: WindowEvent::Resized(..), .. } => {
                window.request_redraw();
            }
            _ => ()
        }
    });
}


fn draw_rect_safe(buffer: &mut [u32], buffer_width: usize, rect: layout::Rect, color: u32) {
    let x0 = rect.x as usize;
    let y0 = rect.y as usize;
    let x1 = (rect.x + rect.width) as usize;
    let y1 = (rect.y + rect.height) as usize;

    let buffer_height = buffer.len() / buffer_width;

    let x0 = x0.clamp(0, buffer_width);
    let x1 = x1.clamp(0, buffer_width);
    let y0 = y0.clamp(0, buffer_height);
    let y1 = y1.clamp(0, buffer_height);

    for y in y0..y1 {
        for x in x0..x1 {
            let index = y * buffer_width + x;
            if index < buffer.len() {
                buffer[index] = color;
            }
        }
    }
}