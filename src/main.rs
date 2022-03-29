use std::{
	collections::VecDeque,
	sync::{atomic::AtomicUsize, RwLock},
	time::Instant,
};

use eframe::{egui, epaint::Color32};
use fastrand::Rng;

lazy_static::lazy_static! {
	static ref TRIES: AtomicUsize = AtomicUsize::new(0);
	static ref RECORD: AtomicUsize = AtomicUsize::new(0);
	static ref THREAD_BEST: RwLock<Vec<(usize, String)>> = RwLock::new({
		let mut out = vec![];
		for _ in 0..num_cpus::get() {
			out.push((0, "".to_string()));
		}
		out
	});
	static ref LETTERS: [char; 28] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', ' ', '\n'];
	static ref GOAL_STRING: Vec<char> = include_str!("goal.txt")
		.chars()
		.collect::<Vec<_>>();
	static ref START_TIME: Instant = Instant::now();
}

fn main() {
	let _threads = (0..num_cpus::get())
		.into_iter()
		.map(|n| std::thread::spawn(move || thread(n)))
		.collect::<Vec<_>>();
	eframe::run_native(Box::new(App), eframe::NativeOptions::default());
}

#[inline(always)]
fn thread(n: usize) {
	let mut buffer = VecDeque::from(vec![' '; GOAL_STRING.len()]);
	let mut record = 0;
	let mut counter = 0;
	let mut rng = Rng::new();
	loop {
		buffer.push_front(rand_char(&mut rng));
		buffer.truncate(GOAL_STRING.len());
		let similarity = compare(&mut buffer.iter(), &mut GOAL_STRING.iter());
		if similarity > record {
			record = similarity;
			RECORD.fetch_max(similarity, std::sync::atomic::Ordering::Relaxed);
			let mut best = THREAD_BEST.write().unwrap();
			best[n] = (record, buffer.iter().collect::<String>());
		}
		counter += 1;
		if counter == 100000 {
			TRIES.fetch_add(100000, std::sync::atomic::Ordering::Relaxed);
			counter = 0;
		}
	}
}

/// Counts the number of common characters between two strings.
#[cfg(feature = "albert")]
fn compare<'a, A, B>(a: &mut A, b: &mut B) -> usize
where
	A: Iterator<Item = &'a char>,
	B: Iterator<Item = &'a char>,
{
	// Albert's comparisons
	a.zip(b)
		.take_while(|(a, b)| **a as u32 == **b as u32)
		.count()
}

#[cfg(not(feature = "albert"))]
fn compare<'a, A, B>(a: &mut A, b: &mut B) -> usize
where
	A: Iterator<Item = &'a char>,
	B: Iterator<Item = &'a char>,
{
	a.zip(b).filter(|(a, b)| **a as u32 == **b as u32).count()
}

#[inline(always)]
fn rand_char(rng: &mut Rng) -> char {
	LETTERS[rng.usize(..LETTERS.len())]
}

#[derive(Default)]
struct App;

impl eframe::epi::App for App {
	fn update(&mut self, ctx: &eframe::egui::Context, _frame: &eframe::epi::Frame) {
		ctx.request_repaint();
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading("the monkeys are hard at work");
			ui.label(format!(
				"current record: {}",
				RECORD.load(std::sync::atomic::Ordering::Relaxed)
			));
			let tries = TRIES.load(std::sync::atomic::Ordering::Relaxed) as f64;
			let time = START_TIME.elapsed().as_secs_f64();
			ui.label(format!(
				"reached after {} tries in {} seconds at {} tries/s",
				human_format::Formatter::new().format(tries),
				time,
				if (tries / time).is_finite() {
					human_format::Formatter::new().format(tries / time)
				} else {
					"???".to_string()
				}
			));
			ui.collapsing("see individual monke stats", |ui| {
				for (n, (count, best)) in THREAD_BEST.read().unwrap().iter().enumerate() {
					let correct: Vec<(char, bool)> = best
						.chars()
						.zip(GOAL_STRING.iter())
						.map(|(a, b)| (a, a == *b))
						.collect();
					ui.horizontal(|ui| {
						ui.label(format!("monke {} got {} right with ", n, count));
						for (c, correct) in correct.iter().take(32) {
							let c = match *c {
								'\n' => ' ',
								c => c,
							};
							if c != ' ' {
								ui.add_space(-7.0);
							} else {
								ui.add_space(-3.0);
							}
							ui.colored_label(
								if *correct {
									Color32::GREEN
								} else {
									Color32::RED
								},
								c.to_string(),
							);
						}
					});
				}
			})
		});
	}

	fn name(&self) -> &str {
		"monke"
	}
}
