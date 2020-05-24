/*!
# FYI Progress Example: Empires of the Ages.

This is a simple threaded progress bar demo, using the list of empires from
`https://en.wikipedia.org/wiki/List_of_empires` to represent tasks, with
lifetimes scaled according to each empire's duration.
*/

/// Do it.
fn main() {
	use fyi_msg::Msg;
	use fyi_progress::Progress;
	use rayon::prelude::*;
	use std::{
		sync::Arc,
		thread,
		time::Duration,
	};



	// The conversion rate for years into milliseconds.
	const RATE: f64 = 1.5;

	// The hard data!
	const DATA: &[(&str, usize)] = &[
		("Elamite Empire", 2500),
		("Akkadian Empire", 100),
		("Assyria", 1119),
		("Babylonian Empire", 300),
		("Egyptian Empire", 473),
		("Mitanni Empire", 200),
		("Hittite Empire", 280),
		("Kingdom of Israel (united monarchy)", 486),
		("Zhou dynasty", 794),
		("Carthaginian Empire", 504),
		("Kushite Empire", 104),
		("Neo-Babylonian Empire", 87),
		("Median Empire", 76),
		("Achaemenid Empire", 220),
		("Nanda Empire", 100),
		("Chera dynasty", 2129),
		("Chola dynasty", 1940),
		("Pandya dynasty", 2159),
		("Macedonian Empire", 11),
		("Mauryan Empire", 136),
		("Seleucid Empire", 249),
		("Ptolemaic Empire", 275),
		("Parthian Empire", 471),
		("Satavahana dynasty", 450),
		("Qin dynasty", 15),
		("Han dynasty", 426),
		("Armenian Empire", 618),
		("Shunga Empire", 112),
		("Dacian Empire", 274),
		("Pontic Empire", 73),
		("Kanva dynasty ", 45),
		("Goguryeo", 705),
		("Roman Empire", 1480),
		("Shatavahana Dynasty", 306),
		("Wadiyar dynasty (Kingdom of Mysore)", 700),
		("Xin dynasty", 14),
		("Kushan Empire", 315),
		("Funan", 500),
		("Aksumite Empire", 790),
		("Cao Wei", 45),
		("Shu Han", 42),
		("Sassanid dynasty", 427),
		("Eastern Wu", 51),
		("Frankish Empire", 700),
		("Gallic Empire", 14),
		("Jin dynasty (265–420)", 155),
		("Palmyrene Empire", 3),
		("Byzantine Empire", 1176),
		("Britannic Empire", 10),
		("Ghana Empire", 940),
		("Gupta Empire", 230),
		("Rouran Khaganate", 225),
		("Hunnic Empire", 99),
		("Western Roman Empire", 81),
		("Hephthalite Empire", 147),
		("Toltec Empire", 626),
		("Wari Empire", 600),
		("Chalukya dynasty", 210),
		("Chenla", 252),
		("Göktürk Khaganate", 195),
		("Sui dynasty", 37),
		("Gurjara-Pratihara dynasty", 660),
		("Empire of Harsha", 41),
		("Tang dynasty", 289),
		("Rashidun Caliphate", 29),
		("Umayyad Caliphate", 89),
		("First Bulgarian Empire", 338),
		("Srivijaya Empire", 610),
		("Republic of Venice", 1100),
		("Balhae", 228),
		("Turgesh Khaganate", 67),
		("Kanem Empire", 687),
		("Khazar Khaganate", 300),
		("Uyghur Khaganate", 106),
		("Abbasid Caliphate", 508),
		("Pala Empire", 424),
		("Rashtrakuta dynasty", 229),
		("Tibetan Empire", 115),
		("Caliphate of Córdoba", 275),
		("Idrisid dynasty", 186),
		("Chauhan dynasty", 400),
		("Khmer Empire", 629),
		("Samanid Empire", 180),
		("Tahirid dynasty", 52),
		("Great Moravian Empire", 67),
		("Kara-Khanid Khanate", 372),
		("Pagan Empire", 448),
		("Saffarid dynasty", 135),
		("Tondo dynasty", 687),
		("Fatimid Caliphate", 262),
		("Liao dynasty", 210),
		("Goryeo", 474),
		("Buyid dynasty", 121),
		("Tu'i Tonga Empire", 915),
		("Holy Roman Empire", 844),
		("Ghaznavid dynasty", 224),
		("Western Chalukya Empire", 216),
		("Georgian Empire", 482),
		("North Sea Empire", 19),
		("Hoysala Empire", 317),
		("Great Seljuq Empire", 157),
		("Western Xia dynasty", 189),
		("Almoravid dynasty", 107),
		("Uyunid Emirate", 163),
		("Khwarazmian dynasty", 144),
		("Jin dynasty (1115–1234)", 119),
		("Almohad Caliphate", 148),
		("Ethiopian Empire", 837),
		("Ghurid dynasty", 67),
		("Angevin Empire", 88),
		("Ayyubid dynasty", 170),
		("Second Bulgarian Empire", 237),
		("Latin Empire", 57),
		("Empire of Nicaea", 57),
		("Empire of Trebizond", 257),
		("Delhi Sultanate", 321),
		("Mongol Empire", 162),
		("Empire of Thessalonica", 42),
		("Chagatai Khanate", 462),
		("Ahom Dynasty", 610),
		("Mali Empire", 375),
		("Tlemcen", 321),
		("Golden Horde", 260),
		("Marinid dynasty", 221),
		("Mamluk Sultanate", 267),
		("Ilkhanate", 79),
		("Yuan dynasty", 97),
		("Cebu Rajahnate ", 286),
		("Khilji dynasty", 30),
		("Majapahit Empire", 234),
		("Ottoman Empire", 623),
		("Vijayanagara Empire", 310),
		("Songhai Empire", 251),
		("Serbian Empire", 25),
		("Jolof Empire", 199),
		("Hanseatic League", 292),
		("Duchy of Burgundy", 113),
		("Bruneian Empire", 520),
		("Ming dynasty", 276),
		("Northern Yuan dynasty", 267),
		("Timurid Empire", 156),
		("Bornu Empire", 506),
		("Kalmar Union", 126),
		("Oyo Empire", 505),
		("Spanish Empire", 573),
		("Portuguese Empire", 584),
		("Duchy of Savoy", 297),
		("Aztec Empire", 93),
		("Later Lê dynasty", 361),
		("Inca Empire (Tawantinsuyo)", 95),
		("Benin Empire", 457),
		("Crimean Khanate", 342),
		("Lodi Sultanate", 75),
		("Safavid dynasty", 235),
		("Akwamu", 362),
		("Toungoo dynasty", 242),
		("Empire of Great Fulo", 262),
		("Mughal Empire", 232),
		("Madurai Nayak dynasty ", 207),
		("Thanjavur Nayak dynasty ", 141),
		("French colonial empire", 486),
		("Danish colonial empire", 417),
		("Kaabu Empire", 330),
		("Saadi dynasty", 105),
		("Dutch Empire", 407),
		("Ramnad Sethupathis ", 389),
		("Gorkha Empire", 250),
		("British Empire", 417),
		("Swedish Empire", 110),
		("Qing dynasty", 268),
		("Commonwealth of England", 11),
		("Rozwi Empire", 206),
		("Ashanti Empire", 232),
		("Maratha Empire", 144),
		("Omani Empire", 260),
		("Lakota people", 177),
		("Kingdom of Prussia", 170),
		("Hotak dynasty", 29),
		("Kong Empire", 298),
		("Bamana Empire", 149),
		("Russian Empire (Romanov)", 196),
		("Sikh Empire", 116),
		("Afsharid Dynasty", 60),
		("Durrani Empire", 75),
		("Zand dynasty", 44),
		("Konbaung dynasty", 133),
		("Tây Sơn dynasty", 24),
		("Siam Empire", 150),
		("Qajar dynasty[citation needed]", 131),
		("Nguyễn dynasty", 143),
		("Austrian Empire", 63),
		("First French Empire", 10),
		("Sokoto Caliphate", 99),
		("Zulu Empire", 79),
		("Massina Empire", 42),
		("First Mexican Empire", 2),
		("Empire of Brazil", 67),
		("Gaza Empire", 71),
		("Toucouleur Empire", 45),
		("Second French Empire", 18),
		("British Raj", 89),
		("Second Mexican Empire", 3),
		("Austria-Hungary", 51),
		("Empire of Japan", 79),
		("German Empire", 47),
		("Wassoulou Empire", 45),
		("Congo Free State", 23),
		("Italian Empire", 58),
		("Korean Empire", 13),
		("Belgian colonial empire", 61),
		("Pahlavi dynasty", 53),
		("Manchukuo", 13),
		("Third Reich", 12),
	];

	// Some titles to change along the way.
	let title1: String = Msg::new("Civilization:", 199, "Reticulating splines…").to_string();
	let title2: String = Msg::new("Civilization:", 199, "*Still* reticulating those splines…").to_string();
	let title3: String = Msg::new("Civilization:", 9, "Approaching the edge of the universe…").to_string();

	// Switch titles at these milestones.
	const START_TITLE2: f64 = 107.0 / 213.0;
	const START_TITLE3: f64 = 193.0 / 213.0;

	// Start a bar.
	let bar = Arc::new(Progress::new(
		DATA.len() as u64,
		Some(title1),
	));

	// Launch a steady tick.
	let handle = Progress::steady_tick(&bar, None);

	// Do our "work".
	DATA.into_par_iter().for_each(|(p, d)| {
		bar.clone().add_task(*p);

		// Simulate processing overhead.
		let pause: u64 = (*d as f64 * RATE) as u64;
		thread::sleep(Duration::from_millis(pause));

		// Being a demonstration, let's show how a title might be changed as
		// certain percentage milestones are reached. Frivolous but fun!
		let percent: f64 = bar.clone().percent();
		if percent == START_TITLE2 {
			bar.clone().update(
				1,
				Some(title2.clone()),
				Some(*p)
			);
		}
		else if percent == START_TITLE3 {
			bar.clone().update(
				1,
				Some(title3.clone()),
				Some(*p)
			);
		}
		else { bar.clone().update(1, None::<String>, Some(*p)); }
	});
	handle.join().unwrap();

	// Print a quick summary.
	// bar.finished_in();
}
