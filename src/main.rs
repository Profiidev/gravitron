use ash::vk;
use glam as g;
use model::TexturedInstanceData;
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalSize, Size},
};

use crate::camera::Camera;
use crate::aetna::Aetna;
use crate::model::Model;

mod camera;
mod debug;
mod surface;
mod swapchain;
mod queues;
mod pipeline;
mod pools;
mod buffer;
mod model;
mod aetna;
mod light;
mod texture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let event_loop = winit::event_loop::EventLoop::new().unwrap();
  event_loop
    .run_app(&mut App {
      aetna: None,
      frame: 0,
      start_time: std::time::Instant::now(),
      camera: Camera::builder().build(),
    })
    .unwrap();

  Ok(())
}

struct App {
  aetna: Option<Aetna>,
  frame: u64,
  start_time: std::time::Instant,
  camera: Camera,
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    let window_attributes = winit::window::WindowAttributes::default()
      .with_title("Vulkan")
      .with_inner_size(Size::Logical(LogicalSize::new(800.0, 600.0)));

    let window = event_loop.create_window(window_attributes).unwrap();
    let mut aetna = Aetna::init(window).unwrap();

    let mut quad = Model::quad();
    
    quad.insert_visibly(TexturedInstanceData::new(g::Mat4::from_translation(g::Vec3::from_array([0.0, 0.0, 0.0]))));

    quad.update_instance_buffer(&mut aetna.allocator, &aetna.device).unwrap();
    //quad.update_texture(&mut aetna.allocator, &aetna.device).unwrap();
    quad.update_index_buffer(&mut aetna.allocator, &aetna.device).unwrap();
    quad.update_vertex_buffer(&mut aetna.allocator, &aetna.device).unwrap();

    let models = vec![quad];
    aetna.models = models;

    self.aetna = Some(aetna);
  }

  fn window_event(
    &mut self,
    _event_loop: &winit::event_loop::ActiveEventLoop,
    _window_id: winit::window::WindowId,
    event: winit::event::WindowEvent,
  ) {
    match event {
      winit::event::WindowEvent::CloseRequested => {
        std::mem::drop(self.aetna.take());
      }
      winit::event::WindowEvent::KeyboardInput {
        device_id: _,
        event:
          winit::event::KeyEvent {
            logical_key: key,
            state: winit::event::ElementState::Pressed,
            ..
          },
        is_synthetic: _,
      } => match key.as_ref() {
        winit::keyboard::Key::Character("w") => {
          self.camera.move_forward(0.05);
        }
        winit::keyboard::Key::Character("s") => {
          self.camera.move_backward(0.05);
        }
        winit::keyboard::Key::Character("a") => {
          self.camera.move_left(0.05);
        }
        winit::keyboard::Key::Character("d") => {
          self.camera.move_right(0.05);
        }
        winit::keyboard::Key::Character("q") => {
          self.camera.move_up(0.05);
        }
        winit::keyboard::Key::Character("e") => {
          self.camera.move_down(0.05);
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
          self.camera.turn_up(0.05);
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
          self.camera.turn_down(0.05);
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft) => {
          self.camera.turn_left(0.05);
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight) => {
          self.camera.turn_right(0.05);
        }
        _ => {}
      },
      winit::event::WindowEvent::RedrawRequested => {
        if let Some(aetna) = self.aetna.as_mut() {
          unsafe {
            aetna
              .device
              .wait_for_fences(
                &[aetna.swapchain.may_begin_drawing[aetna.swapchain.current_image]],
                true,
                std::u64::MAX,
              )
              .expect("Unable to wait for fences");

            aetna
              .device
              .reset_fences(&[aetna.swapchain.may_begin_drawing[aetna.swapchain.current_image]])
              .expect("Unable to reset fences");
          }

          self.camera.update_buffer(&mut aetna.uniform_buffer).unwrap();

          for m in &mut aetna.models {
            m.update_instance_buffer(&mut aetna.allocator, &aetna.device)
              .unwrap();
          }

          aetna
            .update_command_buffer(aetna.swapchain.current_image)
            .expect("Unable to update command buffer");

          let image_index = unsafe {
            match aetna
              .swapchain
              .loader
              .acquire_next_image(
                aetna.swapchain.swapchain,
                std::u64::MAX,
                aetna.swapchain.image_available[aetna.swapchain.current_image],
                vk::Fence::null(),
              ) {
              Ok((image_index, _)) => image_index,
              Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                aetna.recreate_swapchain().expect("Unable to recreate swapchain");
                self.camera.set_aspect_ratio(aetna.swapchain.extent.width as f32 / aetna.swapchain.extent.height as f32);
                self.camera.update_buffer(&mut aetna.uniform_buffer).unwrap();
                return;
              }
              Err(e) => {
                panic!("Unable to acquire next image: {:?}", e);
              }
            }
          };

          let semaphore_available =
            [aetna.swapchain.image_available[aetna.swapchain.current_image]];
          let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
          let semaphore_render_finished =
            [aetna.swapchain.render_finished[aetna.swapchain.current_image]];
          let command_buffer = [aetna.command_buffers[aetna.swapchain.current_image]];

          let submit_info = [vk::SubmitInfo::default()
            .wait_semaphores(&semaphore_available)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffer)
            .signal_semaphores(&semaphore_render_finished)];

          unsafe {
            aetna
              .device
              .queue_submit(
                aetna.queues.graphics,
                &submit_info,
                aetna.swapchain.may_begin_drawing[aetna.swapchain.current_image],
              )
              .expect("Unable to submit queue");
          }

          let swapchains = [aetna.swapchain.swapchain];
          let image_indices = [image_index];
          let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&semaphore_render_finished)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
          match unsafe {
            aetna
              .swapchain
              .loader
              .queue_present(aetna.queues.graphics, &present_info)
          } {
            Ok(_) => {},
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
              aetna.recreate_swapchain().expect("Unable to recreate swapchain");
              self.camera.set_aspect_ratio(aetna.swapchain.extent.width as f32 / aetna.swapchain.extent.height as f32);
              self.camera.update_buffer(&mut aetna.uniform_buffer).unwrap();
            },
            Err(e) => {
              panic!("Unable to present queue: {:?}", e);
            }
          }

          aetna.swapchain.current_image =
            (aetna.swapchain.current_image + 1) % aetna.swapchain.amount_of_images as usize;

          self.frame += 1;
          let elapsed = self.start_time.elapsed();
          if elapsed.as_secs() >= 1 {
            println!("FPS: {}", self.frame);
            self.frame = 0;
            self.start_time = std::time::Instant::now();
          }

          let max_frames = 165;
          let frame_time = std::time::Duration::from_secs(1) / max_frames;
          let elapsed = self.start_time.elapsed();
          if elapsed < frame_time * self.frame as u32 {
            std::thread::sleep(frame_time * self.frame as u32 - elapsed);
          }
        }
      }
      _ => {}
    }
  }

  fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
    if let Some(aetna) = self.aetna.as_mut() {
      aetna.window.request_redraw();
    }
  }

}
