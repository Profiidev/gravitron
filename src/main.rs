use ash::vk;
use glam as g;
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalSize, Size},
};
use gpu_allocator::vulkan;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    self.handle = cube.insert_visibly(InstanceData::new(
      g::Mat4::from_translation(g::Vec3::new(0.5, 0.0, 0.0))
        * g::Mat4::from_scale(g::Vec3::from_array([0.5, 0.01, 0.01])),
      [1.0, 0.5, 0.5],
    ));
    self.handle = cube.insert_visibly(InstanceData::new(
      g::Mat4::from_translation(g::Vec3::new(0.0, 0.5, 0.0))
        * g::Mat4::from_scale(g::Vec3::from_array([0.01, 0.5, 0.01])),
      [0.5, 1.0, 0.5],
    ));
    self.handle = cube.insert_visibly(InstanceData::new(
      g::Mat4::from_translation(g::Vec3::new(0.0, 0.0, 0.5))
        * g::Mat4::from_scale(g::Vec3::from_array([0.01, 0.01, 0.5])),
      [0.5, 0.5, 1.0],
    ));

    let mut ico = Model::sphere(3);
    self.handle = ico.insert_visibly(InstanceData::new(
      g::Mat4::from_scale(g::Vec3::from_array([0.5, 0.5, 0.5])),
      [0.5, 0.0, 0.0],
    ));

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
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::F12) => {
          screenshot(self.aetna.as_mut().unwrap()).expect("Unable to take screenshot");
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

fn screenshot(aetna: &mut Aetna) -> Result<(), Box<dyn std::error::Error>> {
  let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
    .command_pool(aetna.pools.command_pool_graphics)
    .command_buffer_count(1);
  let copy_buffer = unsafe {
    aetna
      .device
      .allocate_command_buffers(&command_buffer_allocate_info)
      .expect("Unable to allocate command buffer")[0]
  };

  let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

  unsafe {
    aetna
      .device
      .begin_command_buffer(copy_buffer, &command_buffer_begin_info)
  }?;

  let ici = vk::ImageCreateInfo::default()
    .image_type(vk::ImageType::TYPE_2D)
    .format(vk::Format::R8G8B8A8_UNORM)
    .extent(vk::Extent3D {
      width: aetna.swapchain.extent.width,
      height: aetna.swapchain.extent.height,
      depth: 1,
    })
    .mip_levels(1)
    .array_layers(1)
    .samples(vk::SampleCountFlags::TYPE_1)
    .tiling(vk::ImageTiling::LINEAR)
    .usage(vk::ImageUsageFlags::TRANSFER_DST)
    .initial_layout(vk::ImageLayout::UNDEFINED);

  let image = unsafe {
    aetna
      .device
      .create_image(&ici, None)
      .expect("Unable to create image")
  };

  let memory_requirements = unsafe {
    aetna
      .device
      .get_image_memory_requirements(image)
  };

  let alloc_info = vulkan::AllocationCreateDesc {
    name: "Screenshot",
    requirements: memory_requirements,
    location: gpu_allocator::MemoryLocation::GpuToCpu,
    linear: true,
    allocation_scheme: vulkan::AllocationScheme::GpuAllocatorManaged,
  };

  let allocation = aetna.allocator.allocate(&alloc_info).unwrap();

  unsafe {
    aetna
      .device
      .bind_image_memory(image, allocation.memory(), allocation.offset())
      .expect("Unable to bind image memory");
  }

  let barrier = vk::ImageMemoryBarrier::default()
    .src_access_mask(vk::AccessFlags::empty())
    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
    .old_layout(vk::ImageLayout::UNDEFINED)
    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
    .image(image)
    .subresource_range(vk::ImageSubresourceRange {
      aspect_mask: vk::ImageAspectFlags::COLOR,
      base_mip_level: 0,
      level_count: 1,
      base_array_layer: 0,
      layer_count: 1,
    });

  unsafe {
    aetna
      .device
      .cmd_pipeline_barrier(
        copy_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[barrier],
      );
  }

  let source_image = aetna.swapchain.images[aetna.swapchain.current_image];
  let barrier = vk::ImageMemoryBarrier::default()
    .src_access_mask(vk::AccessFlags::MEMORY_READ)
    .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
    .old_layout(vk::ImageLayout::PRESENT_SRC_KHR)
    .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
    .image(source_image)
    .subresource_range(vk::ImageSubresourceRange {
      aspect_mask: vk::ImageAspectFlags::COLOR,
      base_mip_level: 0,
      level_count: 1,
      base_array_layer: 0,
      layer_count: 1,
    });

  unsafe {
    aetna
      .device
      .cmd_pipeline_barrier(
        copy_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[barrier],
      );
  }

  let zero_offset = vk::Offset3D::default();
  let copy_region = vk::ImageCopy::default()
    .src_subresource(vk::ImageSubresourceLayers {
      aspect_mask: vk::ImageAspectFlags::COLOR,
      mip_level: 0,
      base_array_layer: 0,
      layer_count: 1,
    })
    .src_offset(zero_offset)
    .dst_subresource(vk::ImageSubresourceLayers {
      aspect_mask: vk::ImageAspectFlags::COLOR,
      mip_level: 0,
      base_array_layer: 0,
      layer_count: 1,
    })
    .dst_offset(zero_offset)
    .extent(vk::Extent3D {
      width: aetna.swapchain.extent.width,
      height: aetna.swapchain.extent.height,
      depth: 1,
    });

  unsafe {
    aetna
      .device
      .cmd_copy_image(
        copy_buffer,
        source_image,
        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        &[copy_region],
      );
  }

  let barrier = vk::ImageMemoryBarrier::default()
    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
    .dst_access_mask(vk::AccessFlags::MEMORY_READ)
    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
    .new_layout(vk::ImageLayout::GENERAL)
    .image(image)
    .subresource_range(vk::ImageSubresourceRange {
      aspect_mask: vk::ImageAspectFlags::COLOR,
      base_mip_level: 0,
      level_count: 1,
      base_array_layer: 0,
      layer_count: 1,
    });

  unsafe {
    aetna
      .device
      .cmd_pipeline_barrier(
        copy_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[barrier],
      );
  }

  let barrier = vk::ImageMemoryBarrier::default()
    .src_access_mask(vk::AccessFlags::TRANSFER_READ)
    .dst_access_mask(vk::AccessFlags::MEMORY_READ)
    .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
    .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
    .image(source_image)
    .subresource_range(vk::ImageSubresourceRange {
      aspect_mask: vk::ImageAspectFlags::COLOR,
      base_mip_level: 0,
      level_count: 1,
      base_array_layer: 0,
      layer_count: 1,
    });

  unsafe {
    aetna
      .device
      .cmd_pipeline_barrier(
        copy_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[barrier],
      );
  }

  unsafe {
    aetna
      .device
      .end_command_buffer(copy_buffer)
  }?;

  let copy = [copy_buffer];
  let submit_info = vk::SubmitInfo::default()
    .command_buffers(&copy);
  let fence = unsafe {
    aetna
      .device
      .create_fence(&vk::FenceCreateInfo::default(), None)
      .expect("Unable to create fence")
  };
  unsafe {
    aetna
      .device
      .queue_submit(aetna.queues.graphics, &[submit_info], fence)
      .expect("Unable to submit queue");
    aetna
      .device
      .wait_for_fences(&[fence], true, std::u64::MAX)
      .expect("Unable to wait for fences");
    aetna
      .device
      .destroy_fence(fence, None);
    aetna
      .device
      .free_command_buffers(aetna.pools.command_pool_graphics, &copy);
  }

  let source_ptr = allocation.mapped_ptr().unwrap().as_ptr() as *mut u8;
  let subresource_layout = unsafe {
    aetna
      .device
      .get_image_subresource_layout(image, vk::ImageSubresource {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        mip_level: 0,
        array_layer: 0,
      })
  };

  let mut data = Vec::with_capacity(subresource_layout.size as usize);
  unsafe {
    std::ptr::copy(
      source_ptr,
      data.as_mut_ptr(),
      subresource_layout.size as usize,
    );
    data.set_len(subresource_layout.size as usize);
  }

  unsafe {
    aetna
      .device
      .destroy_image(image, None);
    aetna
      .allocator
      .free(allocation)
      .unwrap();
  }

  let image = image::ImageBuffer::from_raw(
    aetna.swapchain.extent.width,
    aetna.swapchain.extent.height,
    data,
  ).unwrap();

  let image = image::DynamicImage::ImageRgba8(image);
  image.save("screenshot.png").expect("Unable to save screenshot");
  
  Ok(())
}