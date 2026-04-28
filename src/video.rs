use crate::{Color, HEIGHT, WIDTH};
use std::sync::atomic::Ordering::{AcqRel, Acquire, Release};
use std::sync::atomic::{AtomicBool, AtomicPtr};
use std::sync::Arc;
use web_time::Instant;

type Buffer = [u8; WIDTH as usize * HEIGHT as usize * 4];

pub fn display_triple_buffer() -> (FrontBuffer, BackBuffer) {
    let intermediate_buffer = Arc::new(IntermediateBuffer::default());

    let front = FrontBuffer {
        front_buffer: Some(Box::new([0; _])),
        intermediate_buffer: intermediate_buffer.clone(),
    };

    let back = BackBuffer {
        intermediate_buffer,
        ..Default::default()
    };

    (front, back)
}

pub struct FrontBuffer {
    front_buffer: Option<Box<Buffer>>,
    intermediate_buffer: Arc<IntermediateBuffer>,
}

impl FrontBuffer {
    pub fn read_buffer(&mut self) -> &Buffer {
        if self.intermediate_buffer.dirty.swap(false, AcqRel) {
            let old_buffer = self.front_buffer.take().unwrap();
            self.front_buffer = Some(self.intermediate_buffer.swap(old_buffer));
        }

        self.front_buffer.as_ref().unwrap()
    }
}

#[derive(Debug)]
pub struct BackBuffer {
    back_buffer: Option<Box<Buffer>>,
    offset: usize,
    intermediate_buffer: Arc<IntermediateBuffer>,
    frames_since_fps_log: u64,
    last_fps_log: Instant,
}

impl BackBuffer {
    pub fn write(&mut self, color: Color) {
        let buffer = self.back_buffer.as_mut().unwrap();

        let (r, g, b) = color.to_rgb();
        buffer[self.offset] = r;
        buffer[self.offset + 1] = g;
        buffer[self.offset + 2] = b;
        buffer[self.offset + 3] = 0xFF;

        self.offset += 4;
        if self.offset >= buffer.len() {
            self.offset = 0;
            self.log_fps();
            self.swap_buffers();
        }
    }

    pub fn reset(&mut self) {
        self.offset = 0;
    }

    fn log_fps(&mut self) {
        self.frames_since_fps_log += 1;

        let now = Instant::now();
        let elapsed_seconds = (now - self.last_fps_log).as_secs_f64();
        if elapsed_seconds < 1.0 {
            return;
        }

        let fps = self.frames_since_fps_log as f64 / elapsed_seconds;
        log::info!("FPS: {fps}");
        self.last_fps_log = now;
        self.frames_since_fps_log = 0;
    }

    fn swap_buffers(&mut self) {
        let old_buffer = self.back_buffer.take().unwrap();
        self.back_buffer = Some(self.intermediate_buffer.swap(old_buffer));
        self.intermediate_buffer.dirty.store(true, Release);
    }
}

impl Default for BackBuffer {
    fn default() -> Self {
        Self {
            back_buffer: Some(Box::new([0; _])),
            offset: 0,
            intermediate_buffer: Default::default(),
            frames_since_fps_log: 0,
            last_fps_log: Instant::now(),
        }
    }
}

#[derive(Debug)]
struct IntermediateBuffer {
    buffer: AtomicPtr<Buffer>,
    dirty: AtomicBool,
}

impl IntermediateBuffer {
    fn swap(&self, buffer: Box<Buffer>) -> Box<Buffer> {
        let new_ptr = Box::into_raw(buffer);
        let old_ptr = self.buffer.swap(new_ptr, AcqRel);
        // SAFETY: the pointer is always valid and exclusive
        unsafe { Box::from_raw(old_ptr) }
    }
}

impl Drop for IntermediateBuffer {
    fn drop(&mut self) {
        let ptr = self.buffer.load(Acquire);
        // SAFETY: the pointer is always valid and exclusive
        let boxed = unsafe { Box::from_raw(ptr) };
        drop(boxed)
    }
}

impl Default for IntermediateBuffer {
    fn default() -> Self {
        Self {
            buffer: AtomicPtr::new(Box::into_raw(Box::new([0; _]))),
            dirty: AtomicBool::new(false),
        }
    }
}
