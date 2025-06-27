//! Memory Fragmentation and Garbage Collection Stress Testing
//!
//! This module implements comprehensive testing of memory fragmentation patterns,
//! garbage collection stress scenarios, and analysis of memory allocation efficiency
//! under various fragmentation conditions to ensure system stability.

use sublime_monorepo_tools::Result;
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Configuration for memory fragmentation and GC stress testing
#[derive(Debug, Clone)]
pub struct MemoryFragmentationConfig {
    /// Maximum test duration in seconds
    pub max_test_duration_secs: u64,
    /// Fragmentation analysis interval in milliseconds
    pub analysis_interval_ms: u64,
    /// Base allocation size in KB
    pub base_allocation_size_kb: usize,
    /// Maximum allocation size variation (multiplier)
    pub max_allocation_size_multiplier: f64,
    /// Fragmentation pattern to simulate
    pub fragmentation_pattern: FragmentationPattern,
    /// Target fragmentation level (0.0-1.0)
    pub target_fragmentation_level: f64,
    /// Garbage collection trigger threshold (MB)
    pub gc_trigger_threshold_mb: usize,
    /// Enable automatic garbage collection simulation
    pub enable_gc_simulation: bool,
    /// GC stress intensity (1.0-10.0, higher = more stress)
    pub gc_stress_intensity: f64,
    /// Memory pressure threshold for alerts (MB)
    pub memory_pressure_threshold_mb: usize,
    /// Enable real-time fragmentation monitoring
    pub enable_realtime_monitoring: bool,
}

impl Default for MemoryFragmentationConfig {
    fn default() -> Self {
        Self {
            max_test_duration_secs: 300, // 5 minutes
            analysis_interval_ms: 500,   // 500ms analysis
            base_allocation_size_kb: 64, // 64KB base allocation
            max_allocation_size_multiplier: 8.0, // Up to 512KB allocations
            fragmentation_pattern: FragmentationPattern::RandomSizeAllocations,
            target_fragmentation_level: 0.3, // 30% fragmentation target
            gc_trigger_threshold_mb: 100,   // Trigger GC at 100MB
            enable_gc_simulation: true,
            gc_stress_intensity: 3.0,
            memory_pressure_threshold_mb: 500, // Alert at 500MB
            enable_realtime_monitoring: true,
        }
    }
}

/// Different patterns of memory fragmentation
#[derive(Debug, Clone, PartialEq)]
pub enum FragmentationPattern {
    /// Random size allocations with random deallocations
    RandomSizeAllocations,
    /// Small/large alternating allocations creating gaps
    AlternatingSmallLarge,
    /// Burst allocations followed by partial deallocations
    BurstWithPartialCleanup,
    /// Growing allocation sizes with scattered deallocations
    GrowingWithScatteredFree,
    /// Short-lived vs long-lived object mixing
    ShortLongLivedMixing,
    /// Power-of-2 size allocations creating specific fragmentation
    PowerOfTwoSizes,
    /// Slab-like allocation pattern with fragmentation
    SlabFragmentation,
}

/// Memory fragmentation stress tester
#[derive(Debug)]
pub struct MemoryFragmentationTester {
    /// Configuration for the test
    config: MemoryFragmentationConfig,
    /// Memory allocations tracking
    allocations: Arc<Mutex<HashMap<usize, AllocationInfo>>>,
    /// Fragmentation analysis history
    fragmentation_history: Arc<Mutex<VecDeque<FragmentationAnalysis>>>,
    /// GC stress statistics
    gc_statistics: Arc<Mutex<GarbageCollectionStats>>,
    /// Test control flag
    test_active: Arc<AtomicBool>,
    /// Current allocation ID counter
    allocation_id_counter: Arc<AtomicUsize>,
    /// Total allocated memory (bytes)
    total_allocated_bytes: Arc<AtomicU64>,
    /// Total freed memory (bytes)
    total_freed_bytes: Arc<AtomicU64>,
    /// Memory blocks (simulated allocation tracking)
    memory_blocks: Arc<Mutex<BTreeMap<usize, Vec<u8>>>>,
}

/// Information about a memory allocation
#[derive(Debug, Clone)]
struct AllocationInfo {
    /// Unique allocation identifier
    id: usize,
    /// Size of allocation in bytes
    size_bytes: usize,
    /// Timestamp when allocated
    allocated_at: Instant,
    /// Memory address (simulated)
    simulated_address: usize,
    /// Expected lifetime category
    lifetime_category: AllocationLifetime,
    /// Whether this allocation is still active
    is_active: bool,
}

/// Categories of allocation lifetimes
#[derive(Debug, Clone, PartialEq)]
enum AllocationLifetime {
    /// Very short-lived (< 1 second)
    VeryShort,
    /// Short-lived (1-10 seconds)
    Short,
    /// Medium-lived (10-60 seconds)
    Medium,
    /// Long-lived (> 60 seconds)
    Long,
    /// Permanent (never freed during test)
    Permanent,
}

/// Memory fragmentation analysis result
#[derive(Debug, Clone)]
pub struct FragmentationAnalysis {
    /// Timestamp of analysis
    pub timestamp: Instant,
    /// Fragmentation percentage (0.0-1.0)
    pub fragmentation_percentage: f64,
    /// Total allocated memory (bytes)
    pub total_allocated_bytes: u64,
    /// Total freed memory (bytes)
    pub total_freed_bytes: u64,
    /// Active allocations count
    pub active_allocations_count: usize,
    /// Largest free block size (bytes)
    pub largest_free_block_bytes: usize,
    /// Smallest free block size (bytes)
    pub smallest_free_block_bytes: usize,
    /// Average allocation size (bytes)
    pub average_allocation_size_bytes: f64,
    /// Memory efficiency ratio
    pub memory_efficiency_ratio: f64,
    /// Allocation density (allocations per MB)
    pub allocation_density: f64,
    /// Free space distribution entropy
    pub free_space_entropy: f64,
}

/// Garbage collection statistics
#[derive(Debug, Clone)]
pub struct GarbageCollectionStats {
    /// Total number of GC cycles
    pub total_gc_cycles: usize,
    /// Total time spent in GC (milliseconds)
    pub total_gc_time_ms: u64,
    /// Average GC cycle time (milliseconds)
    pub average_gc_time_ms: f64,
    /// Longest GC pause (milliseconds)
    pub longest_gc_pause_ms: u64,
    /// Total memory reclaimed (bytes)
    pub total_memory_reclaimed_bytes: u64,
    /// GC efficiency (memory reclaimed / GC time)
    pub gc_efficiency_bytes_per_ms: f64,
    /// Number of forced GC cycles
    pub forced_gc_cycles: usize,
    /// Memory pressure incidents
    pub memory_pressure_incidents: usize,
    /// Last GC trigger reason
    pub last_gc_trigger_reason: String,
}

impl Default for GarbageCollectionStats {
    fn default() -> Self {
        Self {
            total_gc_cycles: 0,
            total_gc_time_ms: 0,
            average_gc_time_ms: 0.0,
            longest_gc_pause_ms: 0,
            total_memory_reclaimed_bytes: 0,
            gc_efficiency_bytes_per_ms: 0.0,
            forced_gc_cycles: 0,
            memory_pressure_incidents: 0,
            last_gc_trigger_reason: "None".to_string(),
        }
    }
}

/// Memory fragmentation stress test result
#[derive(Debug, Clone)]
pub struct FragmentationStressResult {
    /// Test duration
    pub test_duration: Duration,
    /// Peak fragmentation level reached
    pub peak_fragmentation_level: f64,
    /// Average fragmentation level
    pub average_fragmentation_level: f64,
    /// Total memory operations performed
    pub total_memory_operations: usize,
    /// Memory efficiency at test end
    pub final_memory_efficiency: f64,
    /// GC performance statistics
    pub gc_performance: GarbageCollectionStats,
    /// Critical fragmentation incidents
    pub critical_fragmentation_incidents: usize,
    /// Memory pressure events
    pub memory_pressure_events: usize,
    /// Fragmentation pattern effectiveness
    pub pattern_effectiveness_score: f64,
    /// Recommended optimizations
    pub recommended_optimizations: Vec<String>,
}

impl MemoryFragmentationTester {
    /// Create a new memory fragmentation tester
    pub fn new(config: MemoryFragmentationConfig) -> Self {
        Self {
            config,
            allocations: Arc::new(Mutex::new(HashMap::new())),
            fragmentation_history: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            gc_statistics: Arc::new(Mutex::new(GarbageCollectionStats::default())),
            test_active: Arc::new(AtomicBool::new(false)),
            allocation_id_counter: Arc::new(AtomicUsize::new(1)),
            total_allocated_bytes: Arc::new(AtomicU64::new(0)),
            total_freed_bytes: Arc::new(AtomicU64::new(0)),
            memory_blocks: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    /// Start memory fragmentation stress test
    pub fn start_fragmentation_test(&self) -> Result<()> {
        self.test_active.store(true, Ordering::SeqCst);

        // Start memory allocation thread
        let alloc_config = self.config.clone();
        let alloc_allocations = Arc::clone(&self.allocations);
        let alloc_counter = Arc::clone(&self.allocation_id_counter);
        let alloc_total_bytes = Arc::clone(&self.total_allocated_bytes);
        let alloc_memory_blocks = Arc::clone(&self.memory_blocks);
        let alloc_active = Arc::clone(&self.test_active);

        thread::spawn(move || {
            Self::run_allocation_thread(
                alloc_config,
                alloc_allocations,
                alloc_counter,
                alloc_total_bytes,
                alloc_memory_blocks,
                alloc_active,
            )
        });

        // Start deallocation thread
        let dealloc_config = self.config.clone();
        let dealloc_allocations = Arc::clone(&self.allocations);
        let dealloc_freed_bytes = Arc::clone(&self.total_freed_bytes);
        let dealloc_memory_blocks = Arc::clone(&self.memory_blocks);
        let dealloc_active = Arc::clone(&self.test_active);

        thread::spawn(move || {
            Self::run_deallocation_thread(
                dealloc_config,
                dealloc_allocations,
                dealloc_freed_bytes,
                dealloc_memory_blocks,
                dealloc_active,
            )
        });

        // Start GC simulation thread
        if self.config.enable_gc_simulation {
            let gc_config = self.config.clone();
            let gc_allocations = Arc::clone(&self.allocations);
            let gc_stats = Arc::clone(&self.gc_statistics);
            let gc_freed_bytes = Arc::clone(&self.total_freed_bytes);
            let gc_memory_blocks = Arc::clone(&self.memory_blocks);
            let gc_active = Arc::clone(&self.test_active);

            thread::spawn(move || {
                Self::run_gc_simulation_thread(
                    gc_config,
                    gc_allocations,
                    gc_stats,
                    gc_freed_bytes,
                    gc_memory_blocks,
                    gc_active,
                )
            });
        }

        // Start fragmentation analysis thread
        if self.config.enable_realtime_monitoring {
            let analysis_config = self.config.clone();
            let analysis_allocations = Arc::clone(&self.allocations);
            let analysis_history = Arc::clone(&self.fragmentation_history);
            let analysis_total_allocated = Arc::clone(&self.total_allocated_bytes);
            let analysis_total_freed = Arc::clone(&self.total_freed_bytes);
            let analysis_active = Arc::clone(&self.test_active);

            thread::spawn(move || {
                Self::run_fragmentation_analysis_thread(
                    analysis_config,
                    analysis_allocations,
                    analysis_history,
                    analysis_total_allocated,
                    analysis_total_freed,
                    analysis_active,
                )
            });
        }

        Ok(())
    }

    /// Stop fragmentation test
    pub fn stop_fragmentation_test(&self) {
        self.test_active.store(false, Ordering::SeqCst);
    }

    /// Run memory allocation thread according to fragmentation pattern
    fn run_allocation_thread(
        config: MemoryFragmentationConfig,
        allocations: Arc<Mutex<HashMap<usize, AllocationInfo>>>,
        allocation_counter: Arc<AtomicUsize>,
        total_allocated_bytes: Arc<AtomicU64>,
        memory_blocks: Arc<Mutex<BTreeMap<usize, Vec<u8>>>>,
        test_active: Arc<AtomicBool>,
    ) {
        let start_time = Instant::now();
        let mut iteration = 0;

        while test_active.load(Ordering::SeqCst) {
            if start_time.elapsed().as_secs() >= config.max_test_duration_secs {
                break;
            }

            let allocation_size = Self::calculate_allocation_size(&config, iteration);
            let lifetime_category = Self::determine_allocation_lifetime(&config, iteration);
            
            if let Ok(allocation_id) = Self::perform_allocation(
                allocation_size,
                lifetime_category,
                &allocations,
                &allocation_counter,
                &total_allocated_bytes,
                &memory_blocks,
            ) {
                // Pattern-specific allocation behavior
                let sleep_duration = Self::get_allocation_sleep_duration(&config.fragmentation_pattern, iteration);
                thread::sleep(sleep_duration);
            }

            iteration += 1;
        }
    }

    /// Calculate allocation size based on pattern and iteration
    fn calculate_allocation_size(config: &MemoryFragmentationConfig, iteration: usize) -> usize {
        let base_size_bytes = config.base_allocation_size_kb * 1024;
        
        match config.fragmentation_pattern {
            FragmentationPattern::RandomSizeAllocations => {
                let multiplier = 1.0 + (iteration % 8) as f64 * 0.5; // 1x to 4.5x
                (base_size_bytes as f64 * multiplier) as usize
            },
            FragmentationPattern::AlternatingSmallLarge => {
                if iteration % 2 == 0 {
                    base_size_bytes / 4 // Small allocation
                } else {
                    base_size_bytes * 4 // Large allocation
                }
            },
            FragmentationPattern::BurstWithPartialCleanup => {
                if iteration % 20 < 15 {
                    base_size_bytes // Normal size during burst
                } else {
                    base_size_bytes / 8 // Tiny allocations during cleanup
                }
            },
            FragmentationPattern::GrowingWithScatteredFree => {
                let growth_factor = 1.0 + (iteration / 50) as f64 * 0.1;
                (base_size_bytes as f64 * growth_factor) as usize
            },
            FragmentationPattern::ShortLongLivedMixing => {
                if iteration % 5 == 0 {
                    base_size_bytes * 2 // Larger long-lived objects
                } else {
                    base_size_bytes / 2 // Smaller short-lived objects
                }
            },
            FragmentationPattern::PowerOfTwoSizes => {
                let power = (iteration % 6) + 1; // 2^1 to 2^6
                base_size_bytes * (1 << power)
            },
            FragmentationPattern::SlabFragmentation => {
                let slab_sizes = [64, 128, 256, 512, 1024, 2048]; // KB
                let slab_index = iteration % slab_sizes.len();
                slab_sizes[slab_index] * 1024 // Convert to bytes
            },
        }
    }

    /// Determine allocation lifetime category
    fn determine_allocation_lifetime(config: &MemoryFragmentationConfig, iteration: usize) -> AllocationLifetime {
        match config.fragmentation_pattern {
            FragmentationPattern::ShortLongLivedMixing => {
                if iteration % 10 == 0 {
                    AllocationLifetime::Long
                } else if iteration % 5 == 0 {
                    AllocationLifetime::Medium
                } else {
                    AllocationLifetime::VeryShort
                }
            },
            FragmentationPattern::BurstWithPartialCleanup => {
                if iteration % 20 < 15 {
                    AllocationLifetime::Short
                } else {
                    AllocationLifetime::VeryShort
                }
            },
            _ => {
                // Default distribution
                match iteration % 10 {
                    0 => AllocationLifetime::Long,
                    1..=2 => AllocationLifetime::Medium,
                    3..=5 => AllocationLifetime::Short,
                    _ => AllocationLifetime::VeryShort,
                }
            }
        }
    }

    /// Perform memory allocation
    fn perform_allocation(
        size_bytes: usize,
        lifetime_category: AllocationLifetime,
        allocations: &Arc<Mutex<HashMap<usize, AllocationInfo>>>,
        allocation_counter: &Arc<AtomicUsize>,
        total_allocated_bytes: &Arc<AtomicU64>,
        memory_blocks: &Arc<Mutex<BTreeMap<usize, Vec<u8>>>>,
    ) -> Result<usize> {
        let allocation_id = allocation_counter.fetch_add(1, Ordering::SeqCst);
        let simulated_address = allocation_id * 1024; // Simulate memory addresses
        
        // Create actual memory allocation for realistic memory pressure
        let memory_block = vec![0u8; size_bytes];
        
        let allocation_info = AllocationInfo {
            id: allocation_id,
            size_bytes,
            allocated_at: Instant::now(),
            simulated_address,
            lifetime_category,
            is_active: true,
        };

        // Store allocation info
        if let Ok(mut allocs) = allocations.lock() {
            allocs.insert(allocation_id, allocation_info);
        }

        // Store memory block
        if let Ok(mut blocks) = memory_blocks.lock() {
            blocks.insert(allocation_id, memory_block);
        }

        total_allocated_bytes.fetch_add(size_bytes as u64, Ordering::SeqCst);
        
        Ok(allocation_id)
    }

    /// Get sleep duration for allocation pattern
    fn get_allocation_sleep_duration(pattern: &FragmentationPattern, iteration: usize) -> Duration {
        match pattern {
            FragmentationPattern::RandomSizeAllocations => Duration::from_millis(10),
            FragmentationPattern::AlternatingSmallLarge => Duration::from_millis(5),
            FragmentationPattern::BurstWithPartialCleanup => {
                if iteration % 20 < 15 {
                    Duration::from_millis(2) // Fast burst
                } else {
                    Duration::from_millis(50) // Slow cleanup
                }
            },
            FragmentationPattern::GrowingWithScatteredFree => Duration::from_millis(15),
            FragmentationPattern::ShortLongLivedMixing => Duration::from_millis(8),
            FragmentationPattern::PowerOfTwoSizes => Duration::from_millis(12),
            FragmentationPattern::SlabFragmentation => Duration::from_millis(6),
        }
    }

    /// Run deallocation thread
    fn run_deallocation_thread(
        config: MemoryFragmentationConfig,
        allocations: Arc<Mutex<HashMap<usize, AllocationInfo>>>,
        total_freed_bytes: Arc<AtomicU64>,
        memory_blocks: Arc<Mutex<BTreeMap<usize, Vec<u8>>>>,
        test_active: Arc<AtomicBool>,
    ) {
        while test_active.load(Ordering::SeqCst) {
            // Find allocations to free based on their lifetime
            let allocations_to_free = Self::identify_allocations_to_free(&allocations);
            
            for allocation_id in allocations_to_free {
                if let Some(freed_bytes) = Self::perform_deallocation(
                    allocation_id,
                    &allocations,
                    &memory_blocks,
                ) {
                    total_freed_bytes.fetch_add(freed_bytes as u64, Ordering::SeqCst);
                }
            }

            thread::sleep(Duration::from_millis(config.analysis_interval_ms / 4));
        }
    }

    /// Identify allocations that should be freed
    fn identify_allocations_to_free(
        allocations: &Arc<Mutex<HashMap<usize, AllocationInfo>>>,
    ) -> Vec<usize> {
        let mut to_free = Vec::new();
        
        if let Ok(allocs) = allocations.lock() {
            let now = Instant::now();
            
            for (allocation_id, info) in allocs.iter() {
                if !info.is_active {
                    continue;
                }
                
                let age = now.duration_since(info.allocated_at);
                let should_free = match info.lifetime_category {
                    AllocationLifetime::VeryShort => age.as_millis() > 500,   // 500ms
                    AllocationLifetime::Short => age.as_secs() > 2,          // 2 seconds
                    AllocationLifetime::Medium => age.as_secs() > 15,        // 15 seconds
                    AllocationLifetime::Long => age.as_secs() > 45,          // 45 seconds
                    AllocationLifetime::Permanent => false,                  // Never freed
                };
                
                if should_free {
                    to_free.push(*allocation_id);
                }
            }
        }
        
        to_free
    }

    /// Perform memory deallocation
    fn perform_deallocation(
        allocation_id: usize,
        allocations: &Arc<Mutex<HashMap<usize, AllocationInfo>>>,
        memory_blocks: &Arc<Mutex<BTreeMap<usize, Vec<u8>>>>,
    ) -> Option<usize> {
        let freed_bytes = if let Ok(mut allocs) = allocations.lock() {
            if let Some(alloc_info) = allocs.get_mut(&allocation_id) {
                alloc_info.is_active = false;
                Some(alloc_info.size_bytes)
            } else {
                None
            }
        } else {
            None
        };

        // Remove memory block
        if let Ok(mut blocks) = memory_blocks.lock() {
            blocks.remove(&allocation_id);
        }

        freed_bytes
    }

    /// Run garbage collection simulation thread
    fn run_gc_simulation_thread(
        config: MemoryFragmentationConfig,
        allocations: Arc<Mutex<HashMap<usize, AllocationInfo>>>,
        gc_statistics: Arc<Mutex<GarbageCollectionStats>>,
        total_freed_bytes: Arc<AtomicU64>,
        memory_blocks: Arc<Mutex<BTreeMap<usize, Vec<u8>>>>,
        test_active: Arc<AtomicBool>,
    ) {
        let mut last_gc_time = Instant::now();
        
        while test_active.load(Ordering::SeqCst) {
            let should_trigger_gc = Self::should_trigger_garbage_collection(
                &config,
                &allocations,
                &memory_blocks,
                last_gc_time,
            );

            if should_trigger_gc {
                let gc_start = Instant::now();
                let (reclaimed_bytes, reclaimed_count) = Self::perform_garbage_collection(
                    &allocations,
                    &memory_blocks,
                    config.gc_stress_intensity,
                );
                let gc_duration = gc_start.elapsed();
                
                // Update GC statistics
                if let Ok(mut stats) = gc_statistics.lock() {
                    Self::update_gc_statistics(&mut stats, gc_duration, reclaimed_bytes, "Automatic");
                }

                total_freed_bytes.fetch_add(reclaimed_bytes as u64, Ordering::SeqCst);
                last_gc_time = Instant::now();
            }

            thread::sleep(Duration::from_millis(100)); // Check every 100ms
        }
    }

    /// Determine if garbage collection should be triggered
    fn should_trigger_garbage_collection(
        config: &MemoryFragmentationConfig,
        allocations: &Arc<Mutex<HashMap<usize, AllocationInfo>>>,
        memory_blocks: &Arc<Mutex<BTreeMap<usize, Vec<u8>>>>,
        last_gc_time: Instant,
    ) -> bool {
        // Trigger based on memory usage
        if let Ok(blocks) = memory_blocks.lock() {
            let total_memory_mb = blocks.values()
                .map(|block| block.len())
                .sum::<usize>() / (1024 * 1024);
                
            if total_memory_mb >= config.gc_trigger_threshold_mb {
                return true;
            }
        }

        // Trigger based on time since last GC
        let time_since_last_gc = Instant::now().duration_since(last_gc_time);
        if time_since_last_gc.as_secs() > 30 {
            return true;
        }

        // Trigger based on fragmentation level
        if let Ok(allocs) = allocations.lock() {
            let inactive_count = allocs.values().filter(|a| !a.is_active).count();
            let total_count = allocs.len();
            
            if total_count > 0 {
                let inactive_ratio = inactive_count as f64 / total_count as f64;
                if inactive_ratio > 0.3 { // 30% inactive allocations
                    return true;
                }
            }
        }

        false
    }

    /// Perform garbage collection
    fn perform_garbage_collection(
        allocations: &Arc<Mutex<HashMap<usize, AllocationInfo>>>,
        memory_blocks: &Arc<Mutex<BTreeMap<usize, Vec<u8>>>>,
        stress_intensity: f64,
    ) -> (usize, usize) {
        let mut reclaimed_bytes = 0;
        let mut reclaimed_count = 0;
        
        // Simulate GC work based on stress intensity
        let gc_work_ms = (stress_intensity * 10.0) as u64;
        thread::sleep(Duration::from_millis(gc_work_ms));

        // Remove inactive allocations
        let inactive_allocations = if let Ok(allocs) = allocations.lock() {
            allocs.iter()
                .filter(|(_, info)| !info.is_active)
                .map(|(id, info)| (*id, info.size_bytes))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        // Remove from allocations map
        if let Ok(mut allocs) = allocations.lock() {
            for (allocation_id, size_bytes) in &inactive_allocations {
                allocs.remove(allocation_id);
                reclaimed_bytes += size_bytes;
                reclaimed_count += 1;
            }
        }

        // Remove from memory blocks
        if let Ok(mut blocks) = memory_blocks.lock() {
            for (allocation_id, _) in &inactive_allocations {
                blocks.remove(allocation_id);
            }
        }

        (reclaimed_bytes, reclaimed_count)
    }

    /// Update garbage collection statistics
    fn update_gc_statistics(
        stats: &mut GarbageCollectionStats,
        gc_duration: Duration,
        reclaimed_bytes: usize,
        trigger_reason: &str,
    ) {
        stats.total_gc_cycles += 1;
        let gc_time_ms = gc_duration.as_millis() as u64;
        stats.total_gc_time_ms += gc_time_ms;
        stats.average_gc_time_ms = stats.total_gc_time_ms as f64 / stats.total_gc_cycles as f64;
        
        if gc_time_ms > stats.longest_gc_pause_ms {
            stats.longest_gc_pause_ms = gc_time_ms;
        }
        
        stats.total_memory_reclaimed_bytes += reclaimed_bytes as u64;
        
        if stats.total_gc_time_ms > 0 {
            stats.gc_efficiency_bytes_per_ms = 
                stats.total_memory_reclaimed_bytes as f64 / stats.total_gc_time_ms as f64;
        }
        
        stats.last_gc_trigger_reason = trigger_reason.to_string();
    }

    /// Run fragmentation analysis thread
    fn run_fragmentation_analysis_thread(
        config: MemoryFragmentationConfig,
        allocations: Arc<Mutex<HashMap<usize, AllocationInfo>>>,
        fragmentation_history: Arc<Mutex<VecDeque<FragmentationAnalysis>>>,
        total_allocated_bytes: Arc<AtomicU64>,
        total_freed_bytes: Arc<AtomicU64>,
        test_active: Arc<AtomicBool>,
    ) {
        while test_active.load(Ordering::SeqCst) {
            let analysis = Self::perform_fragmentation_analysis(
                &allocations,
                total_allocated_bytes.load(Ordering::SeqCst),
                total_freed_bytes.load(Ordering::SeqCst),
            );

            if let Ok(analysis) = analysis {
                if let Ok(mut history) = fragmentation_history.lock() {
                    history.push_back(analysis);
                    if history.len() > 1000 {
                        history.pop_front();
                    }
                }
            }

            thread::sleep(Duration::from_millis(config.analysis_interval_ms));
        }
    }

    /// Perform detailed fragmentation analysis
    fn perform_fragmentation_analysis(
        allocations: &Arc<Mutex<HashMap<usize, AllocationInfo>>>,
        total_allocated_bytes: u64,
        total_freed_bytes: u64,
    ) -> Result<FragmentationAnalysis> {
        let allocations_snapshot = if let Ok(allocs) = allocations.lock() {
            allocs.clone()
        } else {
            return Err(sublime_monorepo_tools::Error::generic(
                "Failed to lock allocations for analysis".to_string()
            ));
        };

        let active_allocations: Vec<&AllocationInfo> = allocations_snapshot
            .values()
            .filter(|info| info.is_active)
            .collect();

        let active_count = active_allocations.len();
        let total_active_bytes: usize = active_allocations
            .iter()
            .map(|info| info.size_bytes)
            .sum();

        // Calculate fragmentation metrics
        let fragmentation_percentage = Self::calculate_fragmentation_percentage(
            &active_allocations,
            total_allocated_bytes,
            total_freed_bytes,
        );

        let (largest_free_block, smallest_free_block) = Self::calculate_free_block_sizes(
            &active_allocations
        );

        let average_allocation_size = if active_count > 0 {
            total_active_bytes as f64 / active_count as f64
        } else {
            0.0
        };

        let memory_efficiency_ratio = if total_allocated_bytes > 0 {
            total_active_bytes as f64 / total_allocated_bytes as f64
        } else {
            0.0
        };

        let allocation_density = if total_active_bytes > 0 {
            active_count as f64 / (total_active_bytes as f64 / (1024.0 * 1024.0))
        } else {
            0.0
        };

        let free_space_entropy = Self::calculate_free_space_entropy(&active_allocations);

        Ok(FragmentationAnalysis {
            timestamp: Instant::now(),
            fragmentation_percentage,
            total_allocated_bytes,
            total_freed_bytes,
            active_allocations_count: active_count,
            largest_free_block_bytes: largest_free_block,
            smallest_free_block_bytes: smallest_free_block,
            average_allocation_size_bytes: average_allocation_size,
            memory_efficiency_ratio,
            allocation_density,
            free_space_entropy,
        })
    }

    /// Calculate fragmentation percentage
    fn calculate_fragmentation_percentage(
        active_allocations: &[&AllocationInfo],
        total_allocated: u64,
        total_freed: u64,
    ) -> f64 {
        if total_allocated == 0 {
            return 0.0;
        }

        // Calculate gaps between allocations (simulated)
        let mut addresses: Vec<usize> = active_allocations
            .iter()
            .map(|info| info.simulated_address)
            .collect();
        addresses.sort_unstable();

        let mut gap_count = 0;
        let mut total_gap_size = 0;

        for i in 1..addresses.len() {
            let gap_size = addresses[i] - addresses[i-1];
            if gap_size > 1024 { // Gaps larger than 1KB
                gap_count += 1;
                total_gap_size += gap_size;
            }
        }

        // Fragmentation is the ratio of gap space to total allocated space
        if total_allocated > 0 {
            (total_gap_size as f64 / total_allocated as f64).min(1.0)
        } else {
            0.0
        }
    }

    /// Calculate free block sizes
    fn calculate_free_block_sizes(active_allocations: &[&AllocationInfo]) -> (usize, usize) {
        if active_allocations.is_empty() {
            return (0, 0);
        }

        let mut addresses: Vec<(usize, usize)> = active_allocations
            .iter()
            .map(|info| (info.simulated_address, info.size_bytes))
            .collect();
        addresses.sort_by_key(|(addr, _)| *addr);

        let mut free_blocks = Vec::new();

        for i in 1..addresses.len() {
            let (prev_addr, prev_size) = addresses[i-1];
            let (curr_addr, _) = addresses[i];
            let gap_size = curr_addr - (prev_addr + prev_size);
            
            if gap_size > 0 {
                free_blocks.push(gap_size);
            }
        }

        if free_blocks.is_empty() {
            (0, 0)
        } else {
            let largest = *free_blocks.iter().max().unwrap_or(&0);
            let smallest = *free_blocks.iter().min().unwrap_or(&0);
            (largest, smallest)
        }
    }

    /// Calculate free space entropy (measure of fragmentation distribution)
    fn calculate_free_space_entropy(active_allocations: &[&AllocationInfo]) -> f64 {
        if active_allocations.len() < 2 {
            return 0.0;
        }

        // Group allocations by size ranges to measure distribution
        let mut size_buckets: HashMap<usize, usize> = HashMap::new();
        
        for info in active_allocations {
            let bucket = (info.size_bytes / 1024).max(1); // KB buckets, minimum 1
            *size_buckets.entry(bucket).or_insert(0) += 1;
        }

        // Calculate entropy
        let total_allocations = active_allocations.len() as f64;
        let mut entropy = 0.0;

        for count in size_buckets.values() {
            let probability = *count as f64 / total_allocations;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }

    /// Get comprehensive fragmentation test results
    pub fn get_fragmentation_results(&self) -> Result<FragmentationStressResult> {
        let fragmentation_history = self.fragmentation_history.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock fragmentation history".to_string())
        })?;

        let gc_stats = self.gc_statistics.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock GC statistics".to_string())
        })?;

        let allocations = self.allocations.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock allocations".to_string())
        })?;

        // Calculate test metrics
        let test_duration = if let Some(first) = fragmentation_history.front() {
            if let Some(last) = fragmentation_history.back() {
                last.timestamp.duration_since(first.timestamp)
            } else {
                Duration::from_secs(0)
            }
        } else {
            Duration::from_secs(0)
        };

        let peak_fragmentation = fragmentation_history
            .iter()
            .map(|analysis| analysis.fragmentation_percentage)
            .fold(0.0, f64::max);

        let average_fragmentation = if !fragmentation_history.is_empty() {
            fragmentation_history
                .iter()
                .map(|analysis| analysis.fragmentation_percentage)
                .sum::<f64>() / fragmentation_history.len() as f64
        } else {
            0.0
        };

        let total_operations = allocations.len();
        
        let final_efficiency = fragmentation_history
            .back()
            .map(|analysis| analysis.memory_efficiency_ratio)
            .unwrap_or(0.0);

        let critical_incidents = fragmentation_history
            .iter()
            .filter(|analysis| analysis.fragmentation_percentage > 0.7)
            .count();

        let memory_pressure_events = fragmentation_history
            .iter()
            .filter(|analysis| {
                (analysis.total_allocated_bytes as f64 / (1024.0 * 1024.0)) > 
                self.config.memory_pressure_threshold_mb as f64
            })
            .count();

        let pattern_effectiveness = Self::calculate_pattern_effectiveness(
            &self.config.fragmentation_pattern,
            peak_fragmentation,
            average_fragmentation,
        );

        let recommended_optimizations = Self::generate_optimization_recommendations(
            peak_fragmentation,
            average_fragmentation,
            &gc_stats,
        );

        Ok(FragmentationStressResult {
            test_duration,
            peak_fragmentation_level: peak_fragmentation,
            average_fragmentation_level: average_fragmentation,
            total_memory_operations: total_operations,
            final_memory_efficiency: final_efficiency,
            gc_performance: gc_stats.clone(),
            critical_fragmentation_incidents: critical_incidents,
            memory_pressure_events,
            pattern_effectiveness_score: pattern_effectiveness,
            recommended_optimizations,
        })
    }

    /// Calculate how effective the fragmentation pattern was
    fn calculate_pattern_effectiveness(
        pattern: &FragmentationPattern,
        peak_fragmentation: f64,
        average_fragmentation: f64,
    ) -> f64 {
        // Each pattern has expected fragmentation characteristics
        let expected_peak = match pattern {
            FragmentationPattern::RandomSizeAllocations => 0.4,
            FragmentationPattern::AlternatingSmallLarge => 0.6,
            FragmentationPattern::BurstWithPartialCleanup => 0.5,
            FragmentationPattern::GrowingWithScatteredFree => 0.7,
            FragmentationPattern::ShortLongLivedMixing => 0.3,
            FragmentationPattern::PowerOfTwoSizes => 0.8,
            FragmentationPattern::SlabFragmentation => 0.4,
        };

        // Effectiveness is how close we got to expected fragmentation
        let peak_effectiveness = 1.0 - (peak_fragmentation - expected_peak).abs();
        let average_effectiveness = 1.0 - (average_fragmentation - expected_peak * 0.7).abs();
        
        ((peak_effectiveness + average_effectiveness) / 2.0).max(0.0).min(1.0)
    }

    /// Generate optimization recommendations
    fn generate_optimization_recommendations(
        peak_fragmentation: f64,
        average_fragmentation: f64,
        gc_stats: &GarbageCollectionStats,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if peak_fragmentation > 0.7 {
            recommendations.push("Critical: Implement memory compaction strategies".to_string());
            recommendations.push("Consider using memory pools for fixed-size allocations".to_string());
        }

        if average_fragmentation > 0.5 {
            recommendations.push("High fragmentation detected - review allocation patterns".to_string());
            recommendations.push("Implement object pooling for frequently allocated types".to_string());
        }

        if gc_stats.average_gc_time_ms > 100.0 {
            recommendations.push("GC pauses are high - optimize collection strategy".to_string());
            recommendations.push("Consider incremental or concurrent garbage collection".to_string());
        }

        if gc_stats.gc_efficiency_bytes_per_ms < 1000.0 {
            recommendations.push("GC efficiency is low - investigate memory layout".to_string());
        }

        if gc_stats.forced_gc_cycles > gc_stats.total_gc_cycles / 4 {
            recommendations.push("Too many forced GC cycles - increase heap size".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Memory management appears optimal".to_string());
            recommendations.push("Continue monitoring for performance regressions".to_string());
        }

        recommendations
    }

    /// Force garbage collection for testing
    pub fn force_garbage_collection(&self) -> Result<(usize, Duration)> {
        let gc_start = Instant::now();
        let (reclaimed_bytes, _) = Self::perform_garbage_collection(
            &self.allocations,
            &self.memory_blocks,
            self.config.gc_stress_intensity,
        );
        let gc_duration = gc_start.elapsed();

        // Update statistics
        if let Ok(mut stats) = self.gc_statistics.lock() {
            stats.forced_gc_cycles += 1;
            Self::update_gc_statistics(&mut stats, gc_duration, reclaimed_bytes, "Forced");
        }

        self.total_freed_bytes.fetch_add(reclaimed_bytes as u64, Ordering::SeqCst);

        Ok((reclaimed_bytes, gc_duration))
    }

    /// Get current fragmentation level
    pub fn get_current_fragmentation_level(&self) -> Result<f64> {
        let fragmentation_history = self.fragmentation_history.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock fragmentation history".to_string())
        })?;

        Ok(fragmentation_history
            .back()
            .map(|analysis| analysis.fragmentation_percentage)
            .unwrap_or(0.0))
    }

    /// Get memory usage statistics
    pub fn get_memory_usage_stats(&self) -> (u64, u64, usize) {
        let total_allocated = self.total_allocated_bytes.load(Ordering::SeqCst);
        let total_freed = self.total_freed_bytes.load(Ordering::SeqCst);
        let active_allocations = if let Ok(allocs) = self.allocations.lock() {
            allocs.values().filter(|info| info.is_active).count()
        } else {
            0
        };

        (total_allocated, total_freed, active_allocations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragmentation_config_creation() {
        let config = MemoryFragmentationConfig::default();
        assert!(config.max_test_duration_secs > 0);
        assert!(config.base_allocation_size_kb > 0);
        assert!(config.target_fragmentation_level > 0.0);
        assert!(config.target_fragmentation_level <= 1.0);
    }

    #[test]
    fn test_fragmentation_tester_creation() {
        let config = MemoryFragmentationConfig::default();
        let tester = MemoryFragmentationTester::new(config);
        
        assert!(!tester.test_active.load(Ordering::SeqCst));
        assert_eq!(tester.allocation_id_counter.load(Ordering::SeqCst), 1);
        assert_eq!(tester.total_allocated_bytes.load(Ordering::SeqCst), 0);
        assert_eq!(tester.total_freed_bytes.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_allocation_size_calculation() {
        let config = MemoryFragmentationConfig {
            fragmentation_pattern: FragmentationPattern::AlternatingSmallLarge,
            base_allocation_size_kb: 64,
            ..Default::default()
        };

        let small_size = MemoryFragmentationTester::calculate_allocation_size(&config, 0);
        let large_size = MemoryFragmentationTester::calculate_allocation_size(&config, 1);
        
        assert!(small_size < large_size);
        assert_eq!(small_size, 64 * 1024 / 4); // Small allocation
        assert_eq!(large_size, 64 * 1024 * 4); // Large allocation
    }

    #[test]
    fn test_allocation_lifetime_determination() {
        let config = MemoryFragmentationConfig {
            fragmentation_pattern: FragmentationPattern::ShortLongLivedMixing,
            ..Default::default()
        };

        let lifetime_0 = MemoryFragmentationTester::determine_allocation_lifetime(&config, 0);
        let lifetime_5 = MemoryFragmentationTester::determine_allocation_lifetime(&config, 5);
        let lifetime_10 = MemoryFragmentationTester::determine_allocation_lifetime(&config, 10);

        assert_eq!(lifetime_10, AllocationLifetime::Long);
        assert_eq!(lifetime_5, AllocationLifetime::Medium);
        assert_eq!(lifetime_0, AllocationLifetime::Long); // 0 % 10 == 0
    }

    #[tokio::test]
    async fn test_fragmentation_integration() -> Result<()> {
        let config = MemoryFragmentationConfig {
            max_test_duration_secs: 2, // Short test
            analysis_interval_ms: 100,
            fragmentation_pattern: FragmentationPattern::RandomSizeAllocations,
            enable_gc_simulation: true,
            ..Default::default()
        };

        let tester = MemoryFragmentationTester::new(config);
        
        // Start test
        tester.start_fragmentation_test()?;
        
        // Wait for some operations
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Check that allocations occurred
        let (total_allocated, total_freed, active_count) = tester.get_memory_usage_stats();
        assert!(total_allocated > 0, "Should have allocated memory");
        assert!(active_count > 0, "Should have active allocations");
        
        // Wait longer for fragmentation analysis
        tokio::time::sleep(Duration::from_millis(1000)).await;
        
        // Check fragmentation level
        let fragmentation_level = tester.get_current_fragmentation_level()?;
        assert!(fragmentation_level >= 0.0);
        assert!(fragmentation_level <= 1.0);
        
        // Force GC
        let (reclaimed_bytes, gc_duration) = tester.force_garbage_collection()?;
        assert!(gc_duration.as_millis() > 0);
        
        // Stop test
        tester.stop_fragmentation_test();
        
        // Get final results
        tokio::time::sleep(Duration::from_millis(100)).await;
        let results = tester.get_fragmentation_results()?;
        
        assert!(results.test_duration.as_millis() > 0);
        assert!(results.total_memory_operations > 0);
        assert!(!results.recommended_optimizations.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_fragmentation_percentage_calculation() {
        let mut allocations = Vec::new();
        
        // Create test allocations with gaps
        allocations.push(&AllocationInfo {
            id: 1,
            size_bytes: 1024,
            allocated_at: Instant::now(),
            simulated_address: 0,
            lifetime_category: AllocationLifetime::Short,
            is_active: true,
        });
        
        allocations.push(&AllocationInfo {
            id: 2,
            size_bytes: 1024,
            allocated_at: Instant::now(),
            simulated_address: 4096, // Gap between 1024 and 4096
            lifetime_category: AllocationLifetime::Short,
            is_active: true,
        });

        let fragmentation = MemoryFragmentationTester::calculate_fragmentation_percentage(
            &allocations,
            8192, // Total allocated
            0,    // Total freed
        );

        assert!(fragmentation > 0.0);
        assert!(fragmentation <= 1.0);
    }

    #[test]
    fn test_pattern_effectiveness_calculation() {
        let effectiveness = MemoryFragmentationTester::calculate_pattern_effectiveness(
            &FragmentationPattern::AlternatingSmallLarge,
            0.6, // Peak fragmentation
            0.4, // Average fragmentation
        );

        assert!(effectiveness >= 0.0);
        assert!(effectiveness <= 1.0);
    }
}