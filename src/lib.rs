pub mod engine;
mod renderer;
mod shaders;
mod vertex;
mod input;

use engine::Engine;
use renderer::Renderer;
use vertex::Vertex;

use std::time::{Duration, Instant};

use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::sync;
use vulkano::sync::{GpuFuture, FlushError};
use vulkano::buffer::TypedBufferAccess;
use vulkano::command_buffer::SubpassContents;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

pub fn begin_loop(mut engine: Engine) {
    // Convert FPS to redraw frequency
    let freq_millis = 1000 / engine.fps;

    let mut recreate_swapchain = false;
    let mut previous_frame_end = Some(sync::now(engine.renderer.device.clone()).boxed());

    let ratio = engine.width / engine.height;

    engine.event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(freq_millis)
        );

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => { *control_flow = ControlFlow::Exit; },

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                engine.width = size.width;
                engine.height = size.height;
                
                let x = engine.width;
                let y = engine.height;

                if x > y {
                    let vx = y*ratio;
                    engine.renderer.viewport.dimensions = [vx as f32, y as f32];
                    engine.renderer.viewport.origin = [((x / 2) - (vx / 2)) as f32, 0.0];
                } else {
                    let vy = x/ratio;
                    engine.renderer.viewport.dimensions = [x as f32, vy as f32];
                    engine.renderer.viewport.origin = [0.0, ((y / 2) - (vy / 2)) as f32];
                }

                engine.renderer.recreate_pipeline().unwrap();
                recreate_swapchain = true
            },

            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input,
                    ..
                },
                ..
            } => {
                engine.input.handle_input(&mut engine.renderer, input);
            }

            Event::WindowEvent {
                event: WindowEvent::MouseInput {
                    state: winit::event::ElementState::Released,
                    ..
                },
                ..
            } => {
                if !engine.input.is_valid_cursor_position() {
                    return;
                }

                let vertices = vec!(
                    Vertex{ position: [engine.input.cursor[0] as f32, engine.input.cursor[1] as f32] },
                    Vertex{ position: [engine.input.cursor[0] as f32 + 0.1, engine.input.cursor[1] as f32] },
                    Vertex{ position: [engine.input.cursor[0] as f32, engine.input.cursor[1] as f32 + 0.1] },
                );

                let vertex_buffer = Renderer::create_polygon(vertices, &engine.renderer.device);
                engine.renderer.add_polygon(vertex_buffer);
            },

            Event::WindowEvent {
                event: WindowEvent::CursorMoved {
                    position,
                    ..
                },
                ..
            } => {
                let real_dims: [f32; 2] = engine.renderer.viewport.dimensions.into();
                let view_dims: [f32; 2] = [
                    real_dims[0] - 2.0 * engine.renderer.viewport.origin[0],
                    real_dims[1] - 2.0 * engine.renderer.viewport.origin[1],
                ];

                let mut pos: [f32; 2] = position.into();
                pos = [
                    (2.0 * (pos[0] - real_dims[0] / 2.0) / real_dims[0]) as f32,
                    (2.0 * (pos[1] - real_dims[1] / 2.0) / real_dims[1]) as f32,
                ];
                pos[0] *= real_dims[0] / view_dims[0];
                pos[1] *= real_dims[1] / view_dims[1];

                engine.input.cursor = pos;
            }

            Event::RedrawEventsCleared => {
                previous_frame_end.as_mut().unwrap().cleanup_finished();

                if recreate_swapchain {
                    if let Err(_) = engine.renderer.recreate_swapchain() {
                        return;
                    }
                    recreate_swapchain = false;
                }

                let (image_num, suboptimal, acquire_future) =
                    match engine.renderer.acquire_next_image() {
                        Ok(d) => d,
                        Err(_) => {
                            recreate_swapchain = true;
                            return;
                        }
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                let clear_values = vec![[0.0, 0.0, 0.0].into()];

                let mut builder = AutoCommandBufferBuilder::primary(
                    engine.renderer.device.clone(),
                    engine.renderer.queue.family(),
                    CommandBufferUsage::OneTimeSubmit,
                ).unwrap();

                builder
                    .begin_render_pass(
                        engine.renderer.framebuffers[image_num].clone(),
                        SubpassContents::Inline,
                        clear_values,
                    )
                    .unwrap()
                    .set_viewport(0, [engine.renderer.viewport.clone()])
                    .bind_pipeline_graphics(engine.renderer.pipeline.clone());

                for buffer in &engine.renderer.vertex_buffers {
                    builder
                        .bind_vertex_buffers(0, buffer.clone())
                        .draw(buffer.len() as u32, 1, 0, 0)
                        .unwrap();
                }

                builder
                    .end_render_pass()
                    .unwrap();

                let command_buffer = builder.build().unwrap();

                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(engine.renderer.queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(engine.renderer.queue.clone(), engine.renderer.swapchain.clone(), image_num)
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    },
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(engine.renderer.device.clone()).boxed());
                    },
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end = Some(sync::now(engine.renderer.device.clone()).boxed());
                    }
                }
            }
            _ => (),
        }
    });
}
