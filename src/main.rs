#[macro_use]
extern crate gfx;
extern crate cgmath;
extern crate fnv;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate gilrs;
extern crate glutin;

use gfx::traits::FactoryExt;
use gfx::Device;
use gfx::Factory;

use glutin::GlContext;

mod aabb;
mod game;

type ColourFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;
type Resources = gfx_device_gl::Resources;

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
const QUAD_COORDS: [[f32; 2]; 4] = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];

const MAX_NUM_QUADS: usize = 1024;
const NUM_QUADS: usize = 2;

gfx_vertex_struct!(QuadCorners {
    corner_zero_to_one: [f32; 2] = "a_CornerZeroToOne",
});

gfx_vertex_struct!(QuadInstance {
    position_within_window_in_pixels: [f32; 2] = "i_PositionWithinWindowInPixels",
    size_in_pixels: [f32; 2] = "i_SizeInPixels",
    colour: [f32; 3] = "i_Colour",
});

gfx_constant_struct!(Properties {
    window_size_in_pixels: [f32; 2] = "u_WindowSizeInPixels",
});

gfx_pipeline!(pipe {
    quad_corners: gfx::VertexBuffer<QuadCorners> = (),
    quad_instances: gfx::InstanceBuffer<QuadInstance> = (),
    properties: gfx::ConstantBuffer<Properties> = "Properties",
    out_colour: gfx::BlendTarget<ColourFormat> =
        ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
});

fn create_instance_buffer<R, F, T>(
    size: usize,
    factory: &mut F,
) -> Result<gfx::handle::Buffer<R, T>, gfx::buffer::CreationError>
where
    R: gfx::Resources,
    F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
{
    factory.create_buffer(
        size,
        gfx::buffer::Role::Vertex,
        gfx::memory::Usage::Data,
        gfx::memory::Bind::TRANSFER_DST,
    )
}

fn main() {
    let (width, height) = (960, 720);
    let builder = glutin::WindowBuilder::new()
        .with_dimensions(width, height)
        .with_min_dimensions(width, height)
        .with_max_dimensions(width, height);
    let mut events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let (window, mut device, mut factory, rtv, dsv) = gfx_window_glutin::init::<
        ColourFormat,
        DepthFormat,
    >(builder, context, &events_loop);

    let pso = factory
        .create_pipeline_simple(
            include_bytes!("shaders/shader.150.vert"),
            include_bytes!("shaders/shader.150.frag"),
            pipe::new(),
        )
        .expect("Failed to create pipeline");
    let mut encoder: gfx::Encoder<Resources, gfx_device_gl::CommandBuffer> =
        factory.create_command_buffer().into();

    let quad_corners_data = QUAD_COORDS
        .iter()
        .map(|v| QuadCorners {
            corner_zero_to_one: *v,
        })
        .collect::<Vec<_>>();

    let (quad_corners_buf, slice) =
        factory.create_vertex_buffer_with_slice(&quad_corners_data, &QUAD_INDICES[..]);

    let data = pipe::Data {
        quad_corners: quad_corners_buf,
        quad_instances: create_instance_buffer(MAX_NUM_QUADS, &mut factory)
            .expect("Failed to create instance buffer"),
        properties: factory.create_constant_buffer(1),
        out_colour: rtv,
        out_depth: dsv,
    };
    let mut bundle = gfx::pso::bundle::Bundle::new(slice, pso, data);

    let (window_width, window_height, _, _) = bundle.data.out_colour.get_dimensions();
    let properties = Properties {
        window_size_in_pixels: [window_width as f32, window_height as f32],
    };

    bundle.slice.instances = Some((NUM_QUADS as u32, 0));
    let quad_instances_upload: gfx::handle::Buffer<Resources, QuadInstance> = factory
        .create_upload_buffer(MAX_NUM_QUADS)
        .expect("Failed to create instance upload buffer");
    {
        let mut quad_instance_writer = factory
            .write_mapping(&quad_instances_upload)
            .expect("Failed to map upload buffer");

        quad_instance_writer[0].position_within_window_in_pixels = [0., 0.];
        quad_instance_writer[0].size_in_pixels = [300., 300.];
        quad_instance_writer[0].colour = [1., 1., 0.];
        quad_instance_writer[1].position_within_window_in_pixels = [300., 500.];
        quad_instance_writer[1].size_in_pixels = [200., 500.];
        quad_instance_writer[1].colour = [0., 1., 1.];
    }
    encoder.update_constant_buffer(&bundle.data.properties, &properties);

    let mut gilrs = gilrs::Gilrs::new().unwrap();

    let mut game_state = game::GameState::new();
    let mut input_model = game::InputModel::default();
    let mut running = true;
    while running {
        while let Some(event) = gilrs.next_event() {
            match event.event {
                gilrs::EventType::AxisChanged(axis, value, _) => match axis {
                    gilrs::ev::Axis::LeftStickX => input_model.set_x(value),
                    gilrs::ev::Axis::LeftStickY => input_model.set_y(-value),
                    _ => (),
                },
                gilrs::EventType::ButtonPressed(button, _)
                | gilrs::EventType::ButtonRepeated(button, _) => match button {
                    gilrs::ev::Button::DPadUp => input_model.set_y(1.),
                    gilrs::ev::Button::DPadDown => input_model.set_y(-1.),
                    gilrs::ev::Button::DPadLeft => input_model.set_x(-1.),
                    gilrs::ev::Button::DPadRight => input_model.set_x(1.),
                    _ => (),
                },
                gilrs::EventType::ButtonChanged(button, value, _) => match button {
                    gilrs::ev::Button::DPadUp => input_model.set_y(value),
                    gilrs::ev::Button::DPadDown => input_model.set_y(-value),
                    gilrs::ev::Button::DPadLeft => input_model.set_x(-value),
                    gilrs::ev::Button::DPadRight => input_model.set_x(value),
                    _ => (),
                },
                gilrs::EventType::ButtonReleased(button, _) => match button {
                    gilrs::ev::Button::DPadUp | gilrs::ev::Button::DPadDown => {
                        input_model.set_y(0.)
                    }
                    gilrs::ev::Button::DPadLeft | gilrs::ev::Button::DPadRight => {
                        input_model.set_x(0.)
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => {
                    running = false;
                }
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        match input.state {
                            glutin::ElementState::Pressed => match virtual_keycode {
                                glutin::VirtualKeyCode::Up => input_model.set_y(-1.),
                                glutin::VirtualKeyCode::Down => input_model.set_y(01.),
                                glutin::VirtualKeyCode::Left => input_model.set_x(-1.),
                                glutin::VirtualKeyCode::Right => input_model.set_x(1.),
                                _ => (),
                            },
                            glutin::ElementState::Released => match virtual_keycode {
                                glutin::VirtualKeyCode::Up => input_model.set_y(0.),
                                glutin::VirtualKeyCode::Down => input_model.set_y(0.),
                                glutin::VirtualKeyCode::Left => input_model.set_x(0.),
                                glutin::VirtualKeyCode::Right => input_model.set_x(0.),
                                _ => (),
                            },
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        });

        game_state.update(&input_model);

        encoder.clear(&bundle.data.out_colour, [0.0, 0.0, 0.0, 1.0]);
        encoder.clear_depth(&bundle.data.out_depth, 1.0);

        let num_quads = {
            let mut quad_instance_writer = factory
                .write_mapping(&quad_instances_upload)
                .expect("Failed to map upload buffer");

            game_state
                .to_render()
                .zip(quad_instance_writer.iter_mut())
                .fold(0, |count, (to_render, writer)| {
                    writer.position_within_window_in_pixels =
                        to_render.aabb.top_left_coord.into();
                    writer.size_in_pixels = to_render.aabb.size.into();
                    writer.colour = to_render.colour;
                    count + 1
                })
        };

        encoder
            .copy_buffer(
                &quad_instances_upload,
                &bundle.data.quad_instances,
                0,
                0,
                num_quads,
            )
            .expect("Failed to copy instances");
        bundle.encode(&mut encoder);

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
