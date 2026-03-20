//! Provides a windowed lock-free ring buffer with a single reader and a single writer.
use std::cell::UnsafeCell;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::Arc;

/// Create a windowed ring buffer with the specified capacity and window sizes.
///
/// The read and write buffers will both initially be zero. This means a reader can immediately
/// access a zero buffer even before any data is written.
pub fn ring_buffer(
    capacity: usize,
    write_window_size: usize,
    read_window_size: usize,
) -> (RingBufferWriter, RingBufferReader) {
    assert!(capacity > 0);
    assert!(write_window_size + read_window_size <= capacity);

    let buffer = (0..capacity)
        .map(|_| UnsafeCell::new(0.0f32))
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let buffer = Arc::new(RingBuffer {
        buffer,
        capacity,
        read_cursor: AtomicUsize::new(0),
        write_cursor: AtomicUsize::new(read_window_size),
    });

    let writer = RingBufferWriter {
        buffer: buffer.clone(),
        window_size: write_window_size,
    };
    let reader = RingBufferReader {
        buffer,
        window_size: read_window_size,
    };

    (writer, reader)
}

/// The writer for a ring buffer.
pub struct RingBufferWriter {
    buffer: Arc<RingBuffer>,
    window_size: usize,
}

impl RingBufferWriter {
    /// Get a mutable slice into the current write window.
    ///
    /// Returns two slices because the buffer may wrap around.
    /// The two slices will always total the window size.
    ///
    /// This method cannot fail, there is always a full window available for writing.
    pub fn get_mut(&mut self) -> (&mut [f32], &mut [f32]) {
        let write_cursor = self.buffer.write_cursor.load(Relaxed);
        // SAFETY: write window is guaranteed to be between read_cursor and read_cursor+capacity
        unsafe { self.buffer.get_mut(write_cursor, self.window_size) }
    }

    /// Advance the writer by the given `count`, if there is space available.
    ///
    /// Any new values will be set to zero. Returns whether operation succeeded.
    #[must_use]
    pub fn next(&mut self, count: usize) -> bool {
        let write_cursor = self.buffer.write_cursor.load(Relaxed);
        let read_cursor = self.buffer.read_cursor.load(Acquire);
        let new_write_cursor = write_cursor + count;

        if new_write_cursor + self.window_size > read_cursor + self.buffer.capacity {
            return false;
        }

        // Fill new values entering write window with 0
        let (s1, s2) = unsafe { self.buffer.get_mut(write_cursor + self.window_size, count) };
        s1.fill(0f32);
        s2.fill(0f32);

        self.buffer.write_cursor.store(new_write_cursor, Release);
        true
    }

    /// Get the current window size.
    pub fn window_size(&self) -> usize {
        self.window_size
    }

    /// Change the window size, if there is space available.
    ///
    /// Any new values will be set to zero. Returns whether operation succeeded.
    #[must_use]
    pub fn set_window_size(&mut self, new_window_size: usize) -> bool {
        let write_cursor = self.buffer.write_cursor.load(Relaxed);
        let read_cursor = self.buffer.read_cursor.load(Acquire);

        if write_cursor + new_window_size > read_cursor + self.buffer.capacity {
            return false;
        }

        // Fill new values entering write window with 0
        if new_window_size > self.window_size {
            let start = write_cursor + self.window_size;
            let increase = new_window_size - self.window_size;
            let (s1, s2) = unsafe { self.buffer.get_mut(start, increase) };
            s1.fill(0f32);
            s2.fill(0f32);
        }

        self.window_size = new_window_size;
        true
    }
}

/// The reader for a ring buffer.
pub struct RingBufferReader {
    buffer: Arc<RingBuffer>,
    window_size: usize,
}

impl RingBufferReader {
    /// Get an immutable slice into the current read window.
    ///
    /// Returns two slices because the buffer may wrap around.
    /// The two slices will always total the window size.
    ///
    /// This method cannot fail, there is always a full window available for reading.
    pub fn get(&self) -> (&[f32], &[f32]) {
        let read_cursor = self.buffer.read_cursor.load(Relaxed);
        // SAFETY: read window is guaranteed to be between write_cursor-capacity and write_cursor
        unsafe { self.buffer.get(read_cursor, self.window_size) }
    }

    /// Advance the reader by the given `count`, if there is space available.
    ///
    /// Returns whether operation succeeded.
    #[must_use]
    pub fn next(&mut self, count: usize) -> bool {
        let write_cursor = self.buffer.write_cursor.load(Acquire);
        let read_cursor = self.buffer.read_cursor.load(Relaxed);
        let new_read_cursor = read_cursor + count;

        if new_read_cursor + self.window_size > write_cursor {
            return false;
        }

        self.buffer.read_cursor.store(new_read_cursor, Release);
        true
    }

    /// Get the current window size.
    pub fn window_size(&self) -> usize {
        self.window_size
    }

    /// Change the window size, if there is space available.
    ///
    /// Returns whether operation succeeded.
    #[must_use]
    pub fn set_window_size(&mut self, new_window_size: usize) -> bool {
        let write_cursor = self.buffer.write_cursor.load(Acquire);
        let read_cursor = self.buffer.read_cursor.load(Relaxed);

        if read_cursor + new_window_size > write_cursor {
            return false;
        }

        self.window_size = new_window_size;
        true
    }
}

struct RingBuffer {
    capacity: usize,
    buffer: Box<[UnsafeCell<f32>]>,
    read_cursor: AtomicUsize,
    write_cursor: AtomicUsize,
}

impl RingBuffer {
    /// # Safety
    /// The range must lie outside the writer's region (write_cursor -> read_cursor + capacity)
    unsafe fn get(&self, start: usize, size: usize) -> (&[f32], &[f32]) {
        let start = start % self.capacity;
        let end = (start + size) % self.capacity;
        let buffer = self.buffer.as_ptr() as *mut f32;

        if end > start || size == 0 {
            let slice = std::slice::from_raw_parts(buffer.add(start), size);
            (slice, &[])
        } else {
            // Window wraps around
            (
                std::slice::from_raw_parts(buffer.add(start), self.capacity - start),
                std::slice::from_raw_parts(buffer, end),
            )
        }
    }

    /// # Safety
    /// The range must lie outside the reader's region (read_cursor -> write_cursor)
    unsafe fn get_mut(&self, start: usize, size: usize) -> (&mut [f32], &mut [f32]) {
        let start = start % self.capacity;
        let end = (start + size) % self.capacity;
        let buffer = self.buffer.as_ptr() as *mut f32;

        if end > start || size == 0 {
            let slice = std::slice::from_raw_parts_mut(buffer.add(start), size);
            (slice, &mut [])
        } else {
            // Window wraps around
            (
                std::slice::from_raw_parts_mut(buffer.add(start), self.capacity - start),
                std::slice::from_raw_parts_mut(buffer, end),
            )
        }
    }
}

// SAFETY: shared access is managed by the atomic cursors through the single reader and writer
unsafe impl Sync for RingBuffer {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Borrow;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn test_ring_buffer_initial_state() {
        let (mut writer, mut reader) = ring_buffer(10, 3, 3);

        assert_eq!(merge(reader.get()), [0.0, 0.0, 0.0],);
        assert_eq!(merge(writer.get_mut()), [0.0, 0.0, 0.0]);
        assert!(!reader.next(1));
        assert!(writer.next(1));
    }

    #[test]
    fn test_ring_buffer_next() {
        let (mut writer, mut reader) = ring_buffer(10, 3, 3);

        // Can't initially advance reader
        assert!(reader.next(0));
        assert!(!reader.next(1));

        // Can advance writer until it would wrap and overlap reader
        assert!(writer.next(0));
        assert!(writer.next(1));
        assert!(writer.next(2));
        assert!(!writer.next(2));
        assert!(writer.next(1));
        assert!(!writer.next(1));
        assert!(writer.next(0));

        // Can advance reader until it would overlap writer
        assert!(reader.next(0));
        assert!(reader.next(1));
        assert!(reader.next(2));
        assert!(!reader.next(2));
        assert!(reader.next(1));
        assert!(!reader.next(1));
        assert!(reader.next(0));
    }

    #[test]
    fn test_ring_buffer_reading_and_writing() {
        let (mut writer, mut reader) = ring_buffer(10, 3, 3);

        let writer_thread = thread::spawn(move || {
            let mut counter = 1.0;

            while counter < 1000.0 {
                let (left, right) = writer.get_mut();
                for val in left.iter_mut().chain(right.iter_mut()) {
                    assert_eq!(*val, 0.0);
                    *val = counter;
                    counter += 1.0;
                }
                let start = Instant::now();
                while !writer.next(3) {
                    assert!(start.elapsed() < Duration::from_millis(100));
                    std::hint::spin_loop();
                }
            }
        });

        let reader_thread = thread::spawn(move || {
            let mut counter = 1.0;

            while counter < 1000.0 {
                let start = Instant::now();
                while !reader.next(3) {
                    assert!(start.elapsed() < Duration::from_millis(100));
                    std::hint::spin_loop();
                }
                let (left, right) = reader.get();
                for val in left.iter().chain(right.iter()) {
                    assert_eq!(*val, counter);
                    counter += 1.0;
                }
            }
        });

        reader_thread.join().unwrap();
        writer_thread.join().unwrap();
    }

    fn merge<B: Borrow<[f32]>>(pair: (B, B)) -> Vec<f32> {
        let left = pair.0.borrow().iter();
        let right = pair.1.borrow().iter();
        left.chain(right).cloned().collect()
    }
}
