use ash::vk;
use glam as g;
use vulkan::Vulkan;
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalSize, Size},
};

use crate::camera::Camera;
use crate::aetna::Aetna;
use crate::model::{Model, InstanceData};

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
mod vulkan;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  Vulkan::init(vulkan::VulkanConfig::default().set_debug(true))?;
  let event_loop = winit::event_loop::EventLoop::new().unwrap();
  event_loop
    .run_app(&mut App {
      aetna: None,
      frame: 0,
      start_time: std::time::Instant::now(),
      handle: 0,
      camera: Camera::builder().build(),
    })
    .unwrap();

  Ok(())
}

struct App {
  aetna: Option<Aetna>,
  frame: u64,
  start_time: std::time::Instant,
  handle: usize,
  camera: Camera,
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    let window_attributes = winit::window::WindowAttributes::default()
      .with_title("Vulkan")
      .with_inner_size(Size::Logical(LogicalSize::new(800.0, 600.0)));

    let window = event_loop.create_window(window_attributes).unwrap();
    let mut aetna = Aetna::init(window).unwrap();

    let mut cube = Model::cube();

    let mut lights = light::LightManager::default();
    lights.add_light(light::DirectionalLight {
      direction: g::Vec3::new(-1.0, 1.0, 0.0),
      illuminance: [10.0, 10.0, 10.0],
    });
    lights.add_light(light::PointLight {
      position: g::Vec3::new(1.5, 0.0, 0.0),
      illuminance: [10.0, 10.0, 10.0],
    });
    lights.add_light(light::PointLight {
      position: g::Vec3::new(1.5, 0.2, 0.0),
      illuminance: [5.0, 5.0, 5.0],
    });
    lights.add_light(light::PointLight {
      position: g::Vec3::new(1.6, -0.2, 0.1),
      illuminance: [5.0, 5.0, 5.0],
    });
    lights.update_buffer(&mut aetna.light_buffer).unwrap();

    self.handle = cube.insert_visibly(InstanceData::new(
      g::Mat4::from_translation(g::Vec3::new(0.5, 0.0, 0.0))
        * g::Mat4::from_scale(g::Vec3::from_array([0.5, 0.01, 0.01])),
      [1.0, 0.5, 0.5],
      0.0,
      1.0
    ));
    self.handle = cube.insert_visibly(InstanceData::new(
      g::Mat4::from_translation(g::Vec3::new(0.0, 0.5, 0.0))
        * g::Mat4::from_scale(g::Vec3::from_array([0.01, 0.5, 0.01])),
      [0.5, 1.0, 0.5],
      0.0,
      1.0
    ));
    self.handle = cube.insert_visibly(InstanceData::new(
      g::Mat4::from_translation(g::Vec3::new(0.0, 0.0, 0.5))
        * g::Mat4::from_scale(g::Vec3::from_array([0.01, 0.01, 0.5])),
      [0.5, 0.5, 1.0],
      0.0,
      1.0
    ));

    let mut ico = Model::sphere(3);
    for i in 0..10 {
      for j in 0..10 {
        self.handle = ico.insert_visibly(InstanceData::new(
          g::Mat4::from_scale(g::Vec3::from_array([0.5, 0.5, 0.5])) * g::Mat4::from_translation(g::Vec3::new(i as f32 - 5.0, j as f32 - 5.0, 10.0)),
          [0.0, 0.0, 0.8],
          i as f32 * 0.1,
          j as f32 * 0.1
        ));
      }
    }

    cube
      .update_vertex_buffer(&mut aetna.allocator, &aetna.device)
      .unwrap();
    cube
      .update_index_buffer(&mut aetna.allocator, &aetna.device)
      .unwrap();
    cube
      .update_instance_buffer(&mut aetna.allocator, &aetna.device)
      .unwrap();

    ico
      .update_vertex_buffer(&mut aetna.allocator, &aetna.device)
      .unwrap();
    ico
      .update_index_buffer(&mut aetna.allocator, &aetna.device)
      .unwrap();
    ico
      .update_instance_buffer(&mut aetna.allocator, &aetna.device)
      .unwrap();

    let models = vec![cube, ico];
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

          let (image_index, _) = unsafe {
            aetna
              .swapchain
              .loader
              .acquire_next_image(
                aetna.swapchain.swapchain,
                std::u64::MAX,
                aetna.swapchain.image_available[aetna.swapchain.current_image],
                vk::Fence::null(),
              )
              .expect("Unable to acquire next image")
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
          unsafe {
            aetna
              .swapchain
              .loader
              .queue_present(aetna.queues.graphics, &present_info)
              .expect("Unable to queue present");
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
