use std::alloc::{GlobalAlloc, Layout, System};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use freyya_core::{Rakaly, SaveRead, SaveReader, read_save_timed};

struct CountingAllocator;

static HEAP_IN_USE: AtomicUsize = AtomicUsize::new(0);
static HEAP_PEAK: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn grew(bytes: usize) {
    let in_use = HEAP_IN_USE.fetch_add(bytes, Ordering::Relaxed) + bytes;
    HEAP_PEAK.fetch_max(in_use, Ordering::Relaxed);
}

fn shrank(bytes: usize) {
    HEAP_IN_USE.fetch_sub(bytes, Ordering::Relaxed);
}

// SAFETY: every call forwards to the System allocator with the caller's own arguments, so the GlobalAlloc contract passes through unchanged; the counters only observe sizes.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller upholds the GlobalAlloc::alloc contract for layout.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            grew(layout.size());
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller upholds the GlobalAlloc::alloc_zeroed contract for layout.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            grew(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: the caller passes a pointer this allocator returned for layout.
        unsafe { System.dealloc(pointer, layout) };
        shrank(layout.size());
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: the caller upholds the GlobalAlloc::realloc contract for pointer, layout, and new_size.
        let moved = unsafe { System.realloc(pointer, layout, new_size) };
        if !moved.is_null() {
            grew(new_size);
            shrank(layout.size());
        }
        moved
    }
}

pub(crate) fn print_measurements(saves: &[PathBuf], rakaly: &Rakaly) -> ExitCode {
    let mut reader = SaveReader::new();
    let mut failed = false;
    for path in saves {
        if !measure_one(path, rakaly, &mut reader) {
            failed = true;
        }
    }
    if failed {
        ExitCode::from(65)
    } else {
        ExitCode::SUCCESS
    }
}

fn measure_one(path: &Path, rakaly: &Rakaly, reader: &mut SaveReader) -> bool {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {}: {error}", path.display());
            return false;
        }
    };
    let baseline = HEAP_IN_USE.load(Ordering::Relaxed);
    HEAP_PEAK.store(baseline, Ordering::Relaxed);
    let started = Instant::now();
    let (snapshot, times) = match read_save_timed(&bytes, rakaly) {
        Ok(read) => read,
        Err(refusal) => {
            eprintln!("Refused {}: {refusal}", path.display());
            return false;
        }
    };
    let total = started.elapsed();
    let peak = HEAP_PEAK.load(Ordering::Relaxed) - baseline;
    let compact = match serde_json::to_vec(&snapshot) {
        Ok(compact) => compact,
        Err(error) => {
            eprintln!("cannot serialize the Snapshot: {error}");
            return false;
        }
    };
    println!("Save:              {}", path.display());
    println!("Save size:         {}", megabytes(bytes.len()));
    println!(
        "Game date:         {} ({})",
        snapshot.game_date, snapshot.player_country.tag
    );
    println!("Check envelope:    {}", millis(times.check));
    println!("Decode (Rakaly):   {}", millis(times.decode));
    println!("Parse to Snapshot: {}", millis(times.parse));
    println!("Total read:        {}", millis(total));
    println!(
        "Peak heap:         {} (freyya only; Rakaly runs as its own process)",
        megabytes(peak)
    );
    println!("Snapshot size:     {} bytes as compact JSON", compact.len());
    print_repeat_reads(&bytes, rakaly, reader);
    println!();
    true
}

fn print_repeat_reads(bytes: &[u8], rakaly: &Rakaly, reader: &mut SaveReader) {
    for attempt in ["First", "Second"] {
        let started = Instant::now();
        let outcome = match reader.read(bytes, rakaly) {
            Ok(SaveRead::New(_)) => "processed".to_owned(),
            Ok(SaveRead::AlreadyRead(_)) => "recognised as already read".to_owned(),
            Err(refusal) => format!("refused ({refusal})"),
        };
        println!(
            "{attempt} SaveReader read: {outcome} in {}",
            millis(started.elapsed())
        );
    }
}

fn millis(duration: Duration) -> String {
    format!("{:.0} ms", duration.as_secs_f64() * 1000.0)
}

fn megabytes(bytes: usize) -> String {
    format!("{:.1} MB", bytes as f64 / 1_000_000.0)
}
