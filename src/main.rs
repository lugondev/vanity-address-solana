use clap::Parser;
use rayon::prelude::*;
use solana_sdk::signature::{Keypair, Signer};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about = "Generate Solana vanity addresses", long_about = None)]
struct Args {
    /// Prefix pattern to match
    #[arg(short, long)]
    prefix: Option<String>,

    /// Suffix pattern to match
    #[arg(short, long)]
    suffix: Option<String>,

    /// Number of addresses to generate (default: 1)
    #[arg(short = 'n', long, default_value = "1")]
    count: usize,

    /// Number of threads to use (default: number of CPU cores)
    #[arg(short, long)]
    threads: Option<usize>,

    /// Case-sensitive matching (default: case-insensitive)
    #[arg(short, long)]
    case_sensitive: bool,

    /// Show attempts per second
    #[arg(short = 'v', long)]
    verbose: bool,
}

struct VanityMatcher {
    prefix: Option<String>,
    suffix: Option<String>,
    case_sensitive: bool,
}

impl VanityMatcher {
    fn new(prefix: Option<String>, suffix: Option<String>, case_sensitive: bool) -> Result<Self, String> {
        if prefix.is_none() && suffix.is_none() {
            return Err("At least one of --prefix or --suffix must be specified".to_string());
        }

        // Validate Base58 characters
        let base58_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        
        let validated_prefix = if let Some(p) = prefix {
            if p.is_empty() {
                return Err("Prefix cannot be empty".to_string());
            }
            for c in p.chars() {
                if !base58_chars.contains(c) {
                    return Err(format!("Invalid Base58 character in prefix: '{}'", c));
                }
            }
            Some(if case_sensitive { p } else { p.to_lowercase() })
        } else {
            None
        };

        let validated_suffix = if let Some(s) = suffix {
            if s.is_empty() {
                return Err("Suffix cannot be empty".to_string());
            }
            for c in s.chars() {
                if !base58_chars.contains(c) {
                    return Err(format!("Invalid Base58 character in suffix: '{}'", c));
                }
            }
            Some(if case_sensitive { s } else { s.to_lowercase() })
        } else {
            None
        };

        Ok(VanityMatcher { 
            prefix: validated_prefix, 
            suffix: validated_suffix,
            case_sensitive,
        })
    }

    fn matches(&self, address: &str) -> bool {
        let compare_address = if self.case_sensitive {
            address.to_string()
        } else {
            address.to_lowercase()
        };

        let mut matched = true;

        if let Some(ref prefix) = self.prefix {
            matched = matched && compare_address.starts_with(prefix);
        }

        if let Some(ref suffix) = self.suffix {
            matched = matched && compare_address.ends_with(suffix);
        }

        matched
    }

    fn description(&self) -> String {
        let sensitivity = if self.case_sensitive {
            "case-sensitive"
        } else {
            "case-insensitive"
        };
        
        match (&self.prefix, &self.suffix) {
            (Some(p), Some(s)) => format!("prefix '{}' and suffix '{}' ({})", p, s, sensitivity),
            (Some(p), None) => format!("prefix '{}' ({})", p, sensitivity),
            (None, Some(s)) => format!("suffix '{}' ({})", s, sensitivity),
            (None, None) => unreachable!(),
        }
    }
}

fn generate_vanity_address(
    matcher: Arc<VanityMatcher>,
    found: Arc<AtomicBool>,
    attempts: Arc<AtomicU64>,
    results: Arc<Mutex<Vec<(Keypair, String)>>>,
    found_count: Arc<AtomicUsize>,
    target_count: usize,
) {
    while !found.load(Ordering::Relaxed) {
        let keypair = Keypair::new();
        let pubkey = keypair.pubkey().to_string();
        
        attempts.fetch_add(1, Ordering::Relaxed);

        if matcher.matches(&pubkey) {
            let mut results_guard = results.lock().unwrap();
            results_guard.push((keypair, pubkey.clone()));
            let current_count = found_count.fetch_add(1, Ordering::Relaxed) + 1;
            drop(results_guard);
            
            if current_count >= target_count {
                found.store(true, Ordering::Relaxed);
            }
        }
    }
}

fn format_secret_key(keypair: &Keypair) -> String {
    format!("{:?}", keypair.to_bytes())
}

fn main() {
    let args = Args::parse();

    // Validate count
    if args.count == 0 {
        eprintln!("Error: count must be at least 1");
        std::process::exit(1);
    }

    // Validate and create matcher
    let matcher = match VanityMatcher::new(args.prefix, args.suffix, args.case_sensitive) {
        Ok(m) => Arc::new(m),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Set up thread pool
    let num_threads = args.threads.unwrap_or_else(num_cpus::get);
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();

    let plural = if args.count > 1 { "addresses" } else { "address" };
    println!("🔍 Searching for {} Solana vanity {} with {}", args.count, plural, matcher.description());
    println!("⚙️  Using {} threads", num_threads);
    println!("⏳ This may take a while...\n");

    let found = Arc::new(AtomicBool::new(false));
    let attempts = Arc::new(AtomicU64::new(0));
    let results: Arc<Mutex<Vec<(Keypair, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let found_count = Arc::new(AtomicUsize::new(0));
    let start_time = Instant::now();

    // Spawn verbose reporting thread if requested
    let verbose_handle = if args.verbose {
        let attempts_clone = Arc::clone(&attempts);
        let found_clone = Arc::clone(&found);
        let found_count_clone = Arc::clone(&found_count);
        let target_count = args.count;
        Some(std::thread::spawn(move || {
            while !found_clone.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_secs(1));
                let current_attempts = attempts_clone.load(Ordering::Relaxed);
                let current_found = found_count_clone.load(Ordering::Relaxed);
                let elapsed = start_time.elapsed().as_secs_f64();
                let rate = current_attempts as f64 / elapsed;
                print!("\r⚡ Attempts: {} | Found: {}/{} | Rate: {:.0} attempts/sec", 
                    current_attempts, current_found, target_count, rate);
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
        }))
    } else {
        None
    };

    // Generate addresses in parallel
    (0..num_threads)
        .into_par_iter()
        .for_each(|_| {
            generate_vanity_address(
                Arc::clone(&matcher),
                Arc::clone(&found),
                Arc::clone(&attempts),
                Arc::clone(&results),
                Arc::clone(&found_count),
                args.count,
            )
        });

    // Wait for verbose thread to finish
    if let Some(handle) = verbose_handle {
        let _ = handle.join();
    }

    let elapsed = start_time.elapsed();
    let total_attempts = attempts.load(Ordering::Relaxed);
    let final_results = results.lock().unwrap();

    if args.verbose {
        println!(); // New line after progress indicator
    }

    if !final_results.is_empty() {
        let plural = if final_results.len() > 1 { "addresses" } else { "address" };
        println!("✅ Found {} vanity {}!\n", final_results.len(), plural);
        
        for (i, (keypair, address)) in final_results.iter().enumerate() {
            if final_results.len() > 1 {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("Address #{}", i + 1);
            }
            println!("📍 Public Key:  {}", address);
            println!("🔑 Private Key: {}", format_secret_key(keypair));
            println!();
        }
        
        if final_results.len() > 1 {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }
        
        println!("📊 Statistics:");
        println!("   Total Attempts: {}", total_attempts);
        println!("   Addresses Found: {}", final_results.len());
        println!("   Time: {:.2}s", elapsed.as_secs_f64());
        println!("   Rate: {:.0} attempts/sec", total_attempts as f64 / elapsed.as_secs_f64());
        println!("   Avg per address: {:.0} attempts", total_attempts as f64 / final_results.len() as f64);
        
        println!("\n⚠️  IMPORTANT: Save your private keys securely!");
        println!("   You can import them using: solana-keygen recover");
    } else {
        println!("❌ Search interrupted");
    }
}

// Helper function to get number of CPUs
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }
}
