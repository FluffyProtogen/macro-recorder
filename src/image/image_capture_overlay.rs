use glium::*;

use glium::{glutin, Surface};
use glium::{index::PrimitiveType, vertex, Display};
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::platform::windows::WindowBuilderExtWindows;
use winit_input_helper::WinitInputHelper;

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_transparent(true)
        .with_decorations(false)
        .with_maximized(true)
        .with_skip_taskbar(true);
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let program = program!(&display,
        140 => {
            vertex: "
                #version 140
                in vec2 position;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",

            fragment: "
                #version 140
                out vec4 f_color;
                void main() {
                    f_color = vec4(0.05, 0.05, 0.05, 0.1);
                }
            "
        }
    )
    .unwrap();

    let mut input = WinitInputHelper::new();

    let mut previous_mouse_position = (0.0, 0.0);

    let mut initial_mouse_position = None;

    event_loop.run(move |event, _, control_flow| {
        if input.update(&event) {
            if input.mouse_pressed(0) {
                initial_mouse_position = Some(input.mouse().unwrap());
            }

            if input.mouse_released(0) {
                *control_flow = ControlFlow::Exit;
            }
        }

        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::Focused(false) = event {
                *control_flow = ControlFlow::Exit;
            }
        }

        let current_mouse_position = if initial_mouse_position.is_none() {
            None
        } else {
            Some(input.mouse().unwrap_or(previous_mouse_position))
        };

        if let Some(position) = input.mouse() {
            previous_mouse_position = position;
        }

        let half_screen_width = display.get_framebuffer_dimensions().0 as f32 / 2.0;
        let half_screen_height = display.get_framebuffer_dimensions().1 as f32 / 2.0;

        let vertex_buffer = {
            #[derive(Copy, Clone)]
            struct Vertex {
                position: [f32; 2],
            }

            implement_vertex!(Vertex, position);

            if let (Some(current_mouse_position), Some(initial_mouse_position)) =
                (current_mouse_position, initial_mouse_position)
            {
                glium::VertexBuffer::new(
                    &display,
                    &[
                        Vertex {
                            position: [
                                -1.0,
                                -greater(
                                    -1.0 + initial_mouse_position.1 / half_screen_height,
                                    -1.0 + current_mouse_position.1 / half_screen_height,
                                ),
                            ],
                        },
                        Vertex {
                            position: [-1.0, -1.0],
                        },
                        Vertex {
                            position: [
                                1.0,
                                -greater(
                                    -1.0 + initial_mouse_position.1 / half_screen_height,
                                    -1.0 + current_mouse_position.1 / half_screen_height,
                                ),
                            ],
                        },
                        Vertex {
                            position: [1.0, -1.0],
                        },
                        /* */
                        Vertex {
                            position: [-1.0, 1.0],
                        },
                        Vertex {
                            position: [
                                -1.0,
                                greater(
                                    1.0 - initial_mouse_position.1 / half_screen_height,
                                    1.0 - current_mouse_position.1 / half_screen_height,
                                ),
                            ],
                        },
                        Vertex {
                            position: [1.0, 1.0],
                        },
                        Vertex {
                            position: [
                                1.0,
                                greater(
                                    1.0 - initial_mouse_position.1 / half_screen_height,
                                    1.0 - current_mouse_position.1 / half_screen_height,
                                ),
                            ],
                        },
                        /* */
                        Vertex {
                            position: [-1.0, -1.0],
                        },
                        Vertex {
                            position: [
                                -greater(
                                    1.0 - initial_mouse_position.0 / half_screen_width,
                                    1.0 - current_mouse_position.0 / half_screen_width,
                                ),
                                -1.0,
                            ],
                        },
                        Vertex {
                            position: [-1.0, 1.0],
                        },
                        Vertex {
                            position: [
                                -greater(
                                    1.0 - initial_mouse_position.0 / half_screen_width,
                                    1.0 - current_mouse_position.0 / half_screen_width,
                                ),
                                1.0,
                            ],
                        },
                        /* */
                        Vertex {
                            position: [1.0, -1.0],
                        },
                        Vertex {
                            position: [
                                greater(
                                    -1.0 + initial_mouse_position.0 / half_screen_width,
                                    -1.0 + current_mouse_position.0 / half_screen_width,
                                ),
                                -1.0,
                            ],
                        },
                        Vertex {
                            position: [1.0, 1.0],
                        },
                        Vertex {
                            position: [
                                greater(
                                    -1.0 + initial_mouse_position.0 / half_screen_width,
                                    -1.0 + current_mouse_position.0 / half_screen_width,
                                ),
                                1.0,
                            ],
                        },
                    ],
                )
                .unwrap()
            } else {
                glium::VertexBuffer::new(
                    &display,
                    &[
                        Vertex {
                            position: [-1.0, -1.0],
                        },
                        Vertex {
                            position: [-1.0, 1.0],
                        },
                        Vertex {
                            position: [1.0, 1.0],
                        },
                        Vertex {
                            position: [1.0, -1.0],
                        },
                    ],
                )
                .unwrap()
            }
        };

        let index_buffer = if initial_mouse_position.is_some() && current_mouse_position.is_some() {
            glium::IndexBuffer::new(
                &display,
                PrimitiveType::TrianglesList,
                &[
                    0u16, 1, 2, 2, 1, 3, /* */ 4, 5, 6, 6, 5, 7, /* */ 8, 9, 10, 10, 9,
                    11, /* */ 12, 13, 14, 14, 13, 15,
                ],
            )
            .unwrap()
        } else {
            glium::IndexBuffer::new(
                &display,
                PrimitiveType::TrianglesList,
                &[0u16, 1, 2, 2, 3, 0],
            )
            .unwrap()
        };

        draw(
            &display,
            &vertex_buffer,
            &index_buffer,
            &program,
            &uniform!(),
        );
    });
}

fn draw<'a, 'b, V, I, U>(
    display: &Display,
    vertex_buffer: V,
    index_buffer: I,
    program: &Program,
    uniforms: &U,
) where
    I: Into<index::IndicesSource<'a>>,
    U: uniforms::Uniforms,
    V: vertex::MultiVerticesSource<'b>,
{
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target
        .draw(
            vertex_buffer,
            index_buffer,
            &program,
            uniforms,
            &Default::default(),
        )
        .unwrap();
    target.finish().unwrap();
}

fn greater(one: f32, two: f32) -> f32 {
    if one > two {
        one
    } else {
        two
    }
}
