use std::{
	collections::{BTreeMap, VecDeque},
	sync::{atomic::AtomicUsize, Arc, RwLock},
};

use eframe::{egui, epaint::Color32};
use rand::Rng;

lazy_static::lazy_static! {
	static ref TRIES: AtomicUsize = AtomicUsize::new(0);
	static ref RECORD: AtomicUsize = AtomicUsize::new(0);
	static ref THREAD_BEST: Arc<RwLock<BTreeMap<usize, (usize, String)>>> = Arc::new(RwLock::new(BTreeMap::new()));
	static ref LETTERS: [char; 27] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', ' '];
	static ref GOAL_STRING: Vec<char> = "never gonna give you up"
		.chars()
		.collect::<Vec<_>>();
}

fn main() {
	let threads = (0..num_cpus::get())
		.into_iter()
		.map(|n| std::thread::spawn(move || thread(n)))
		.collect::<Vec<_>>();
	eframe::run_native(Box::new(App), eframe::NativeOptions::default());
}

fn thread(n: usize) {
	let mut buffer = VecDeque::from(vec![' '; GOAL_STRING.len()]);
	let mut record = 0;
	let mut rng = rand::thread_rng();
	while record < GOAL_STRING.len() {
		buffer.pop_back();
		buffer.push_front(rand_char(&mut rng));
		let similarity = compare(&mut buffer.iter(), &mut GOAL_STRING.iter());
		if similarity > record {
			record = similarity;
			RECORD.fetch_max(similarity, std::sync::atomic::Ordering::Relaxed);
			let mut best = THREAD_BEST.write().unwrap();
			best.insert(n, (record, buffer.iter().collect::<String>()));
		}
		TRIES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
	}
}

/// Counts the number of common characters between two strings.
fn compare<'a, A, B>(a: &mut A, b: &mut B) -> usize
where
	A: Iterator<Item = &'a char>,
	B: Iterator<Item = &'a char>,
{
	a.zip(b).filter(|(a, b)| a == b).count()
}

fn rand_char(rng: &mut rand::rngs::ThreadRng) -> char {
	*rng.sample(rand::distributions::Slice::new(&*LETTERS).unwrap())
}

#[derive(Default)]
struct App;

impl eframe::epi::App for App {
	fn update(&mut self, ctx: &eframe::egui::Context, frame: &eframe::epi::Frame) {
		ctx.request_repaint();
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading("the monkeys are hard at work");
			ui.label(format!(
				"current record: {}",
				RECORD.load(std::sync::atomic::Ordering::Relaxed)
			));
			ui.label(format!(
				"reached after {} tries",
				human_format::Formatter::new()
					.format(TRIES.load(std::sync::atomic::Ordering::Relaxed) as f64)
			));
			ui.collapsing("see individual monke stats", |ui| {
				for (n, (count, best)) in THREAD_BEST.read().unwrap().iter() {
					let correct: Vec<(char, bool)> = best
						.chars()
						.zip(GOAL_STRING.iter())
						.map(|(a, b)| (a, a == *b))
						.collect();
					ui.horizontal(|ui| {
						ui.label(format!("monke {} got {} right with ", n, count));
						for (c, correct) in correct.iter() {
							if *c != ' ' {
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
